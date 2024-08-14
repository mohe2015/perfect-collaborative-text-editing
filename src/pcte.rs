// runtime borrow checking or handles or raw pointers
use std::fmt::Debug;
use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Index, IndexMut},
    ptr,
    rc::Rc,
};

use tracing::{info, trace};

use crate::handle_vec::{Handle, HandleVec};
use crate::history::{DAGHistory, History};

#[derive(Debug)]
pub enum Message {
    Insert(InsertMessage),
    Delete(DeleteMessage),
}

#[derive(Debug)]
pub struct InsertMessage {
    pub left_replica_id: Rc<String>,
    pub left_counter: usize,
    pub right_replica_id: Rc<String>,
    pub right_counter: usize,
    pub replica_id: Rc<String>,
    pub counter: usize,
    pub character: char,
}

#[derive(Debug)]
pub struct DeleteMessage {
    pub replica_id: Rc<String>,
    pub counter: usize,
}

#[derive(Debug)]
pub struct Pcte {
    pub replica_id: Rc<String>,
    pub counter: usize,
    pub history: DAGHistory<Message>,
    pub nodes: HandleVec<PcteNode>,
    pub tree_nodes: HandleVec<PcteTreeNode>,
    pub id_to_node: HashMap<(Rc<String>, usize), (Handle<PcteTreeNode>, Handle<PcteTreeNode>)>,
    pub left_origin_tree: Handle<PcteTreeNode>,
    pub right_origin_tree: Handle<PcteTreeNode>,
}

#[derive(Debug, Clone)]
pub struct PcteNode {
    pub replica_id: Rc<String>,
    pub counter: usize,
    pub character: Option<char>,
}

/// we need a handle here because the left and right tree both reference the same object

/// we need a handle here because our map and the tree reference the same tree nodes

#[derive(Debug)]
pub struct PcteTreeNode {
    pub node_handle: Handle<PcteNode>,
    pub children: Vec<Handle<PcteTreeNode>>,
}

impl Pcte {
    pub fn new(replica_id: Rc<String>) -> Self {
        let root_replica_id = Rc::new(String::new());
        let mut nodes = HandleVec::new();
        let handle = nodes.push(PcteNode {
            character: None,
            replica_id: root_replica_id.clone(),
            counter: 0,
        });
        let mut tree_nodes = HandleVec::new();
        let left_origin_tree = tree_nodes.push(PcteTreeNode {
            node_handle: handle,
            children: Vec::new(),
        });
        let right_origin_tree = tree_nodes.push(PcteTreeNode {
            node_handle: handle,
            children: Vec::new(),
        });
        Self {
            replica_id,
            counter: 0,
            history: DAGHistory::new(),
            nodes,
            tree_nodes,
            id_to_node: HashMap::from([(
                (root_replica_id, 0),
                (left_origin_tree, right_origin_tree),
            )]),
            left_origin_tree,
            right_origin_tree,
        }
    }

    pub fn insert(&mut self, index: usize, character: char) {
        #[cfg(debug_assertions)]
        let mut text = self.text();

        self.counter += 1;

        let node = PcteNode {
            character: Some(character),
            replica_id: self.replica_id.clone(),
            counter: self.counter,
        };
        let node_handle = self.nodes.push(node);

        let right_origin = match self.node_at_index(self.left_origin_tree, index) {
            Ok(v) => {
                self.node_last_node_and_index_including_deleted_of_node(
                    self.right_origin_tree,
                    self.tree_nodes[v].node_handle,
                    0,
                )
                .unwrap()
                .0
            }
            Err(_) => self.right_origin_tree,
        };

        let handle = self.tree_nodes.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
        self.tree_nodes[right_origin].children.push(handle);

        let right_replica_id = self.nodes[self.tree_nodes[right_origin].node_handle]
            .replica_id
            .clone();
        let right_counter = self.nodes[self.tree_nodes[right_origin].node_handle].counter;

        let left_origin = if index == 0 {
            self.left_origin_tree
        } else {
            self.node_at_index(self.left_origin_tree, index - 1)
                .unwrap()
        };

        let handle = self.tree_nodes.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
        self.tree_nodes[left_origin].children.push(handle);

        self.history.add_value(Message::Insert(InsertMessage {
            left_replica_id: self.nodes[self.tree_nodes[left_origin].node_handle]
                .replica_id
                .clone(),
            left_counter: self.nodes[self.tree_nodes[left_origin].node_handle].counter,
            right_replica_id,
            right_counter,
            replica_id: self.replica_id.clone(),
            counter: self.counter,
            character: character,
        }));

        /*println!(
            "left origin: {:?}, index: {}, value: {}, right origin: {:?}",
            dbg, index, character, dbg2,
        );*/

        #[cfg(debug_assertions)]
        text.insert(index, character);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.text(), text, "{:#?}", self);
    }

    fn insert_remote(&mut self, insert: &InsertMessage) {
        let node = PcteNode {
            character: Some(insert.character),
            replica_id: insert.replica_id.clone(),
            counter: insert.counter,
        };
        let node_handle = self.nodes.push(node);

        let (left_origin, _) = self
            .id_to_node
            .get(&(insert.left_replica_id.clone(), insert.left_counter))
            .unwrap();

        let (_, right_origin) = self
            .id_to_node
            .get(&(insert.right_replica_id.clone(), insert.right_counter))
            .unwrap();

        let left_tree_node_handle = self.tree_nodes.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
        let right_tree_node_handle = self.tree_nodes.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });

        self.tree_nodes[left_origin]
            .children
            .push(left_tree_node_handle);
        self.tree_nodes[right_origin]
            .children
            .push(right_tree_node_handle);

        self.id_to_node.insert(
            (insert.replica_id.clone(), insert.counter),
            (left_tree_node_handle, right_tree_node_handle),
        );

        // TODO calculate position at which it was inserted
    }

    pub fn delete(&mut self, index: usize) {
        #[cfg(debug_assertions)]
        let mut text = self.text();

        let node = self.node_at_index(self.left_origin_tree, index).unwrap();

        /*println!(
            "delete {} {:?}",
            index, self.nodes[node.node_handle.0].character
        );*/

        self.history.add_value(Message::Delete(DeleteMessage {
            replica_id: self.nodes[self.tree_nodes[node].node_handle]
                .replica_id
                .clone(),
            counter: self.nodes[self.tree_nodes[node].node_handle].counter,
        }));

        self.nodes[self.tree_nodes[node].node_handle].character = None;

        #[cfg(debug_assertions)]
        text.remove(index);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.text(), text, "{:#?}", self);
    }

    fn delete_remote(&mut self, delete: &DeleteMessage) {
        let (left_origin, right_origin) = self
            .id_to_node
            .get(&(delete.replica_id.clone(), delete.counter))
            .unwrap();

        debug_assert_eq!(
            self.tree_nodes[left_origin].node_handle,
            self.tree_nodes[right_origin].node_handle
        );

        self.nodes[self.tree_nodes[left_origin].node_handle].character = None;
    }

    pub fn text(&mut self) -> String {
        self.text_tree_node(self.left_origin_tree)
    }

    pub fn synchronize(&mut self, other: &mut Self) {
        let new_for_self = other.history.new_for_other(&self.history);
        let new_for_other = self.history.new_for_other(&other.history);

        info!("to self {:?}", &new_for_self);
        for new_self in new_for_self {
            match &new_self.0.value {
                Message::Insert(insert) => self.insert_remote(insert),
                Message::Delete(delete) => self.delete_remote(delete),
            }
            self.history.add_entry(new_self);
        }

        info!("to other {:?}", &new_for_other);
        for new_other in new_for_other {
            match &new_other.0.value {
                Message::Insert(insert) => other.insert_remote(insert),
                Message::Delete(delete) => other.delete_remote(delete),
            }
            other.history.add_entry(new_other);
        }
    }

    fn text_tree_node(&self, this: Handle<PcteTreeNode>) -> String {
        let mut result = String::new();
        if let Some(character) = self.nodes[self.tree_nodes[this].node_handle].character {
            result.push(character);
        }
        let mut children: Vec<_> = self.tree_nodes[this].children.clone();
        children.sort_by_cached_key(|element| {
            (
                -isize::try_from(
                    self.node_last_node_and_index_including_deleted_of_node(
                        self.right_origin_tree,
                        self.tree_nodes[*element].node_handle,
                        0,
                    )
                    .unwrap()
                    .1,
                )
                .unwrap(),
                self.nodes[self.tree_nodes[element].node_handle]
                    .replica_id
                    .clone(),
                self.nodes[self.tree_nodes[element].node_handle].counter,
            )
        });
        for child in children {
            result.push_str(&self.text_tree_node(child))
        }
        result
    }

    fn node_at_index(
        &mut self,
        node: Handle<PcteTreeNode>,
        mut index: usize,
    ) -> Result<Handle<PcteTreeNode>, usize> {
        if let Some(_) = self.nodes[self.tree_nodes[node].node_handle].character {
            if index == 0 {
                return Ok(node);
            }
            index -= 1;
        }
        let mut children: Vec<_> = self.tree_nodes[node].children.clone();
        children.sort_by_cached_key(|element| {
            (
                -isize::try_from(
                    self.node_last_node_and_index_including_deleted_of_node(
                        self.right_origin_tree,
                        self.tree_nodes[*element].node_handle,
                        0,
                    )
                    .unwrap()
                    .1,
                )
                .unwrap(),
                self.nodes[self.tree_nodes[element].node_handle]
                    .replica_id
                    .clone(),
                self.nodes[self.tree_nodes[element].node_handle].counter,
            )
        });
        for child in children {
            match self.node_at_index(child, index) {
                Ok(ok) => return Ok(ok),
                Err(new_index) => {
                    index = new_index;
                }
            }
        }
        Err(index)
    }

    /// Returns `Ok(index)` if the node is found and `Err(size)` if the node is not found.
    pub fn node_last_node_and_index_including_deleted_of_node(
        &self,
        this: Handle<PcteTreeNode>,
        node: Handle<PcteNode>,
        mut index: usize,
    ) -> Result<(Handle<PcteTreeNode>, usize), usize> {
        if self.tree_nodes[this].node_handle == node {
            return Ok((this, index));
        }
        index += 1;
        for child in &self.tree_nodes[this].children {
            // chldren at the same place need to return the same value
            if self.tree_nodes[child].node_handle == node {
                return Ok((*child, index));
            }
        }
        for child in &self.tree_nodes[this].children {
            match self.node_last_node_and_index_including_deleted_of_node(*child, node, index) {
                ok @ Ok(_) => return ok,
                Err(new_index) => {
                    index = new_index;
                }
            }
        }
        Err(index)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut pcte = Pcte::new(Rc::new("a".to_string()));
        pcte.insert(0, 'h');
        pcte.insert(1, 'e');
        pcte.insert(2, 'l');
        pcte.insert(3, 'l');
        pcte.insert(4, 'o');
        let text = pcte.text();
        assert_eq!(text, "hello");
        println!("{:#?}", pcte);
    }

    #[test]
    fn it_works2() {
        let mut pcte = Pcte::new(Rc::new("a".to_string()));
        pcte.insert(0, 'h');
        pcte.delete(0);
        let text = pcte.text();
        assert_eq!(text, "");
        println!("{:#?}", pcte);
    }

    #[test]
    fn it_works3() {
        let mut pcte = Pcte::new(Rc::new("a".to_string()));
        pcte.insert(0, 'o');
        pcte.insert(0, 'l');
        println!("{:#?}", pcte);
        assert_eq!(pcte.text(), "lo");
        pcte.delete(0);
        let text = pcte.text();
        assert_eq!(text, "o");
    }

    #[test]
    fn it_works4() {
        let mut pcte_a = Pcte::new(Rc::new("a".to_string()));
        let mut pcte_b = Pcte::new(Rc::new("b".to_string()));
        pcte_a.insert(0, 'a');
        pcte_b.insert(0, 'b');
        assert_eq!(pcte_a.text(), "a");
        assert_eq!(pcte_b.text(), "b");
        pcte_a.synchronize(&mut pcte_b);
        assert_eq!(pcte_a.text(), "ab", "{:#?}", pcte_a);
        assert_eq!(pcte_b.text(), "ab");
    }
}
