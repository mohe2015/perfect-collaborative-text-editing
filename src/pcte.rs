// runtime borrow checking or handles or raw pointers

#[derive(Debug)]
pub struct Pcte {
    pub left_origin_tree: PcteTreeNode,
    pub right_origin_tree: PcteTreeNode,
}

#[derive(Debug, Clone)]
pub struct PcteNode {
    pub character: Option<char>,
}

#[derive(Debug)]
pub struct PcteTreeNode {
    pub node: PcteNode,
    pub children: Vec<PcteTreeNode>,
}

impl Pcte {
    pub fn new() -> Self {
        let left_root_node = PcteNode { character: None };
        let right_root_node = PcteNode { character: None };
        Self {
            left_origin_tree: PcteTreeNode {
                node: left_root_node,
                children: Vec::new(),
            },
            right_origin_tree: PcteTreeNode {
                node: right_root_node,
                children: Vec::new(),
            },
        }
    }

    pub fn insert(&mut self, index: usize, node: PcteNode) {
        let left_origin = self
            .left_origin_tree
            .node_first_node_at_index(index)
            .unwrap();
        let right_origin = self
            .right_origin_tree
            .node_last_node_at_index(index)
            .unwrap();

        left_origin.children.push(PcteTreeNode {
            node: node.clone(), // TODO FIXME
            children: Vec::new(),
        });
        right_origin.children.push(PcteTreeNode {
            node: node.clone(), // TODO FIXME
            children: Vec::new(),
        });
    }
}

impl PcteTreeNode {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut pcte = Pcte::new();
        pcte.insert(
            0,
            PcteNode {
                character: Some('h'),
            },
        );
        pcte.insert(
            1,
            PcteNode {
                character: Some('e'),
            },
        );
        pcte.insert(
            2,
            PcteNode {
                character: Some('l'),
            },
        );
        pcte.insert(
            3,
            PcteNode {
                character: Some('l'),
            },
        );
        pcte.insert(
            4,
            PcteNode {
                character: Some('o'),
            },
        );
        panic!("{:#?}", pcte);
    }
}
