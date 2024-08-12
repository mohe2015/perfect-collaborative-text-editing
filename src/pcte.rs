// runtime borrow checking or handles or raw pointers

use std::ptr;

#[derive(Debug)]
pub struct Pcte {
    pub nodes: Vec<PcteNode>,
    pub left_origin_tree: PcteTreeNode,
    pub right_origin_tree: PcteTreeNode,
}

#[derive(Debug, Clone)]
pub struct PcteNode {
    pub character: Option<char>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PcteNodeHandle(usize);

#[derive(Debug)]
pub struct PcteTreeNode {
    pub node_handle: PcteNodeHandle,
    pub children: Vec<PcteTreeNode>,
}

impl Pcte {
    pub fn new() -> Self {
        let nodes = vec![PcteNode { character: None }];
        Self {
            nodes,
            left_origin_tree: PcteTreeNode {
                node_handle: PcteNodeHandle(0),
                children: Vec::new(),
            },
            right_origin_tree: PcteTreeNode {
                node_handle: PcteNodeHandle(0),
                children: Vec::new(),
            },
        }
    }

    pub fn insert(&mut self, index: usize, character: char) {
        #[cfg(debug_assertions)]
        let mut text = self.text();

        let node = PcteNode {
            character: Some(character),
        };
        let node_handle = PcteNodeHandle(self.nodes.len());
        self.nodes.push(node);

        let right_origin = match self.left_origin_tree.node_at_index(
            &mut self.nodes,
            &mut self.right_origin_tree,
            index,
        ) {
            Ok(v) => {
                self.right_origin_tree
                    .node_last_node_and_index_including_deleted_of_node(
                        &self.nodes,
                        v.node_handle,
                        0,
                    )
                    .unwrap()
                    .0
            }
            Err(_) => &mut self.right_origin_tree,
        };

        right_origin.children.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });

        let dbg2 = self.nodes[right_origin.node_handle.0].character;

        let left_origin = if index == 0 {
            &mut self.left_origin_tree
        } else {
            self.left_origin_tree
                .node_at_index(&mut self.nodes, &mut self.right_origin_tree, index - 1)
                .unwrap()
        };

        let dbg = self.nodes[left_origin.node_handle.0].character;

        left_origin.children.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });

        /*println!(
            "left origin: {:?}, index: {}, value: {}, right origin: {:?}",
            dbg, index, character, dbg2,
        );*/

        #[cfg(debug_assertions)]
        text.insert(index, character);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.text(), text, "{:#?}", self);
    }

    pub fn delete(&mut self, index: usize) {
        #[cfg(debug_assertions)]
        let mut text = self.text();

        let node = self
            .left_origin_tree
            .node_at_index(&mut self.nodes, &mut self.right_origin_tree, index)
            .unwrap();

        /*println!(
            "delete {} {:?}",
            index, self.nodes[node.node_handle.0].character
        );*/

        self.nodes[node.node_handle.0].character = None;

        #[cfg(debug_assertions)]
        text.remove(index);
        #[cfg(debug_assertions)]
        debug_assert_eq!(self.text(), text, "{:#?}", self);
    }

    pub fn text(&mut self) -> String {
        self.left_origin_tree
            .text_tree_node(&self.nodes, &mut self.right_origin_tree)
    }
}

impl PcteTreeNode {
    fn text_tree_node(
        self: &PcteTreeNode,
        nodes: &Vec<PcteNode>,
        right_origin_tree: &mut PcteTreeNode,
    ) -> String {
        let mut result = String::new();
        if let Some(character) = nodes[self.node_handle.0].character {
            result.push(character);
        }
        let mut children: Vec<_> = self.children.iter().collect();
        children.sort_by_cached_key(|element| {
            -isize::try_from(
                right_origin_tree
                    .node_last_node_and_index_including_deleted_of_node(
                        nodes,
                        element.node_handle,
                        0,
                    )
                    .unwrap()
                    .1,
            )
            .unwrap()
        });
        for child in children {
            result.push_str(&child.text_tree_node(nodes, right_origin_tree))
        }
        result
    }

    fn node_at_index(
        &mut self,
        nodes: &mut Vec<PcteNode>,
        right_origin_tree: &mut PcteTreeNode,
        mut index: usize,
    ) -> Result<&mut PcteTreeNode, usize> {
        if let Some(_) = nodes[self.node_handle.0].character {
            if index == 0 {
                return Ok(self);
            }
            index -= 1;
        }
        let mut children: Vec<_> = self.children.iter_mut().collect();
        children.sort_by_cached_key(|element| {
            -isize::try_from(
                right_origin_tree
                    .node_last_node_and_index_including_deleted_of_node(
                        nodes,
                        element.node_handle,
                        0,
                    )
                    .unwrap()
                    .1,
            )
            .unwrap()
        });
        for child in children {
            match child.node_at_index(nodes, right_origin_tree, index) {
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
        &mut self,
        nodes: &Vec<PcteNode>,
        element: PcteNodeHandle,
        mut index: usize,
    ) -> Result<(&mut PcteTreeNode, usize), usize> {
        if self.node_handle == element {
            return Ok((self, index));
        }
        index += 1;
        for child in &mut self.children {
            match child.node_last_node_and_index_including_deleted_of_node(nodes, element, index) {
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
        let mut pcte = Pcte::new();
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
        let mut pcte = Pcte::new();
        pcte.insert(0, 'h');
        pcte.delete(0);
        let text = pcte.text();
        assert_eq!(text, "");
        println!("{:#?}", pcte);
    }

    #[test]
    fn it_works3() {
        let mut pcte = Pcte::new();
        pcte.insert(0, 'o');
        pcte.insert(0, 'l');
        println!("{:#?}", pcte);
        assert_eq!(pcte.text(), "lo");
        pcte.delete(0);
        let text = pcte.text();
        assert_eq!(text, "o");
    }
}
