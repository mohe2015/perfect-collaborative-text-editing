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

#[derive(Debug, Clone, Copy)]
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

    fn get_node(&self, handle: PcteNodeHandle) -> &PcteNode {
        &self.nodes[handle.0]
    }

    pub fn insert(&mut self, index: usize, character: char) {
        let node = PcteNode {
            character: Some(character),
        };
        let node_handle = PcteNodeHandle(self.nodes.len());
        self.nodes.push(node);
        let left_origin = self
            .left_origin_tree
            .node_first_node_at_index(index)
            .unwrap();
        let right_origin = self
            .right_origin_tree
            .node_last_node_at_index(index)
            .unwrap();

        left_origin.children.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
        right_origin.children.push(PcteTreeNode {
            node_handle,
            children: Vec::new(),
        });
    }

    pub fn delete(&mut self, index: usize) {
        assert_eq!(
            self.left_origin_tree.delete_internal(
                &mut self.nodes,
                &mut self.right_origin_tree,
                index
            ),
            None
        );
    }

    pub fn text(&self) -> String {
        self.text_tree_node(&self.left_origin_tree)
    }

    fn text_tree_node(&self, tree_node: &PcteTreeNode) -> String {
        let mut result = String::new();
        if let Some(character) = self.get_node(tree_node.node_handle).character {
            result.push(character);
        }
        let mut children: Vec<_> = tree_node.children.iter().collect();
        children
            .sort_by_cached_key(|element| self.right_origin_tree.node_last_index_of_node(element));
        for child in children {
            result.push_str(&self.text_tree_node(child))
        }
        result
    }
}

impl PcteTreeNode {
    fn delete_internal(
        &mut self,
        nodes: &mut Vec<PcteNode>,
        right_origin_tree: &PcteTreeNode,
        mut index: usize,
    ) -> Option<usize> {
        if let Some(_) = nodes[self.node_handle.0].character {
            if index == 0 {
                // TODO FIXME this doesn't delete in the right origin tree
                nodes[self.node_handle.0].character = None;
                return None;
            }
            index -= 1;
        }
        let mut children: Vec<_> = self.children.iter_mut().collect();
        children.sort_by_cached_key(|element| right_origin_tree.node_last_index_of_node(element));
        for child in children {
            if let Some(new_index) = child.delete_internal(nodes, right_origin_tree, index) {
                index = new_index;
            } else {
                return None;
            }
        }
        Some(index)
    }

    /// Returns `Ok(node)` if node is found and `Err(new_index)` if node is not found.
    pub fn node_first_node_at_index(
        &mut self,
        mut index: usize,
    ) -> Result<&mut PcteTreeNode, usize> {
        if index == 0 {
            Ok(self)
        } else {
            index -= 1;
            for child in &mut self.children {
                match child.node_first_node_at_index(index) {
                    Ok(ok) => return Ok(ok),
                    Err(new_index) => {
                        index = new_index;
                    }
                };
            }
            Err(index)
        }
    }

    /// Returns `Ok(node)` if node is found and `Err(new_index)` if node is not found.
    pub fn node_last_node_at_index<'a>(
        &'a mut self,
        mut index: usize,
    ) -> Result<&'a mut PcteTreeNode, usize> {
        for child in &mut self.children {
            match child.node_first_node_at_index(index) {
                Ok(ok) => return Ok(ok),
                Err(new_index) => {
                    index = new_index;
                }
            };
        }
        if index == 0 {
            Ok(self)
        } else {
            index -= 1;
            Err(index)
        }
    }

    /// Returns `Ok(index)` if the node is found and `Err(size)` if the node is not found.
    pub fn node_last_index_of_node(&self, element: &PcteTreeNode) -> Result<usize, usize> {
        let mut index = 0;
        for child in &self.children {
            match child.node_last_index_of_node(element) {
                Ok(result) => return Ok(index + result),
                Err(result) => {
                    index += result;
                }
            }
        }
        if ptr::eq(self, element) {
            return Ok(index);
        }
        unreachable!()
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
        assert_eq!(pcte.text(), "lo");
        pcte.delete(0);
        let text = pcte.text();
        assert_eq!(text, "o");
        println!("{:#?}", pcte);
    }
}
