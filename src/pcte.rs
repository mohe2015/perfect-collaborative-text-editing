// runtime borrow checking or handles or raw pointers

pub struct Pcte {
    pub left_origin_tree: PcteTreeNode,
    pub right_origin_tree: PcteTreeNode,
}

pub struct PcteNode {
    character: char,
}

pub struct PcteTreeNode {
    pub node: PcteNode,
    pub children: Vec<PcteTreeNode>
}