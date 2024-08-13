// runtime borrow checking or handles or raw pointers

use std::{collections::HashMap, hash::Hash, ptr, rc::Rc};

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
    pub nodes: Vec<PcteNode>,
    pub tree_nodes: Vec<PcteTreeNode>,
    pub id_to_node: HashMap<(Rc<String>, usize), (PcteTreeNodeHandle, PcteTreeNodeHandle)>,
    pub left_origin_tree: PcteTreeNodeHandle,
    pub right_origin_tree: PcteTreeNodeHandle,
}

#[derive(Debug, Clone)]
pub struct PcteNode {
    pub replica_id: Rc<String>,
    pub counter: usize,
    pub character: Option<char>,
}

/// we need a handle here because the left and right tree both reference the same object
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcteNodeHandle(usize);

/// we need a handle here because our map and the tree reference the same tree nodes
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcteTreeNodeHandle(usize);

#[derive(Debug)]
pub struct PcteTreeNode {
    pub node_handle: PcteNodeHandle,
    pub children: Vec<PcteTreeNodeHandle>,
}

impl Pcte {
    pub fn new(replica_id: Rc<String>) -> Self {
        let root_replica_id = Rc::new(String::new());
        let nodes = vec![PcteNode {
            character: None,
            replica_id: root_replica_id.clone(),
            counter: 0,
        }];
        let tree_nodes = vec![
            PcteTreeNode {
                node_handle: PcteNodeHandle(0),
                children: Vec::new(),
            },
            PcteTreeNode {
                node_handle: PcteNodeHandle(0),
                children: Vec::new(),
            },
        ];
        Self {
            replica_id,
            counter: 0,
            history: DAGHistory::new(),
            nodes,
            tree_nodes,
            id_to_node: HashMap::from([(
                (root_replica_id, 0),
                (PcteTreeNodeHandle(0), PcteTreeNodeHandle(1)),
            )]),
            left_origin_tree: PcteTreeNodeHandle(0),
            right_origin_tree: PcteTreeNodeHandle(1),
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
        let node_handle = PcteNodeHandle(self.nodes.len());
        self.nodes.push(node);

        let right_origin = match self.node_at_index(self.left_origin_tree, index) {
            Ok(v) => {
                self.node_last_node_and_index_including_deleted_of_node(
                    self.right_origin_tree,
                    self.tree_nodes[v.0].node_handle,
                    0,
                )
                .unwrap()
                .0
            }
            Err(_) => self.right_origin_tree,
        };

        let index = self.tree_nodes.len();
        self.tree_nodes.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
        self.tree_nodes[right_origin.0]
            .children
            .push(PcteTreeNodeHandle(index));

        let right_replica_id = self.nodes[self.tree_nodes[right_origin.0].node_handle.0]
            .replica_id
            .clone();
        let right_counter = self.nodes[self.tree_nodes[right_origin.0].node_handle.0].counter;

        let dbg2 = self.nodes[self.tree_nodes[right_origin.0].node_handle.0].character;

        let left_origin = if index == 0 {
            self.left_origin_tree
        } else {
            self.node_at_index(self.left_origin_tree, index - 1)
                .unwrap()
        };

        let dbg = self.nodes[self.tree_nodes[left_origin.0].node_handle.0].character;

        let index = self.tree_nodes.len();
        self.tree_nodes.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
        self.tree_nodes[left_origin.0]
            .children
            .push(PcteTreeNodeHandle(index));

        self.history.add_value(Message::Insert(InsertMessage {
            left_replica_id: self.nodes[self.tree_nodes[left_origin.0].node_handle.0]
                .replica_id
                .clone(),
            left_counter: self.nodes[self.tree_nodes[left_origin.0].node_handle.0].counter,
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

        todo!("index from replica id and counter to the node");
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
            replica_id: self.nodes[self.tree_nodes[node.0].node_handle.0]
                .replica_id
                .clone(),
            counter: self.nodes[self.tree_nodes[node.0].node_handle.0].counter,
        }));

        self.nodes[self.tree_nodes[node.0].node_handle.0].character = None;

        #[cfg(debug_assertions)]
        text.remove(index);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.text(), text, "{:#?}", self);
    }

    pub fn text(&mut self) -> String {
        self.text_tree_node(self.left_origin_tree)
    }

    pub fn synchronize(&mut self, other: &mut Self) {
        let new_for_self = other.history.new_for_other(&self.history);
        let new_for_other = self.history.new_for_other(&other.history);

        for new_self in new_for_self {
            match &new_self.0.value {
                Message::Insert(insert) => self.insert_remote(insert),
                Message::Delete(delete) => todo!(),
            }
            self.history.add_entry(new_self);
        }

        for new_other in new_for_other {
            other.history.add_entry(new_other);
        }
    }

    fn text_tree_node(&self, this: PcteTreeNodeHandle) -> String {
        let mut result = String::new();
        if let Some(character) = self.nodes[self.tree_nodes[this.0].node_handle.0].character {
            result.push(character);
        }
        let mut children: Vec<_> = self.tree_nodes[this.0].children.iter().collect();
        children.sort_by_cached_key(|element| {
            -isize::try_from(
                self.node_last_node_and_index_including_deleted_of_node(
                    self.right_origin_tree,
                    self.tree_nodes[element.0].node_handle,
                    0,
                )
                .unwrap()
                .1,
            )
            .unwrap()
        });
        for child in children {
            result.push_str(&self.text_tree_node(*child))
        }
        result
    }

    fn node_at_index(
        &mut self,
        node: PcteTreeNodeHandle,
        mut index: usize,
    ) -> Result<PcteTreeNodeHandle, usize> {
        if let Some(_) = self.nodes[self.tree_nodes[node.0].node_handle.0].character {
            if index == 0 {
                return Ok(node);
            }
            index -= 1;
        }
        let mut children: Vec<_> = self.tree_nodes[node.0].children.clone();
        children.sort_by_cached_key(|element| {
            -isize::try_from(
                self.node_last_node_and_index_including_deleted_of_node(
                    self.right_origin_tree,
                    self.tree_nodes[element.0].node_handle,
                    0,
                )
                .unwrap()
                .1,
            )
            .unwrap()
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
        this: PcteTreeNodeHandle,
        node: PcteNodeHandle,
        mut index: usize,
    ) -> Result<(PcteTreeNodeHandle, usize), usize> {
        if self.tree_nodes[this.0].node_handle == node {
            return Ok((this, index));
        }
        index += 1;
        for child in &self.tree_nodes[this.0].children {
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
}
