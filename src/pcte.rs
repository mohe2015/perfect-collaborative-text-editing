// runtime borrow checking or handles or raw pointers

pub struct Pcte {
    pub left_origin_tree: PcteTreeNode,
    pub right_origin_tree: PcteTreeNode,
}

pub struct PcteNode {
    pub character: Option<char>,
}

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

    pub fn insert(&mut self, index: usize, node: PcteNode) {}
}

impl PcteTreeNode {
    pub fn node_first_node_at_index(&self, mut index: usize) -> Result<&PcteTreeNode, usize> {
        if (index == 0) {
            Ok(self)
        } else {
            index -= 1;
            for child in &self.children {
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

    pub fn node_last_node_at_index(&self, index: usize) -> &PcteTreeNode {}
}
