use std::rc::Rc;

pub trait History<T> {
    fn new() -> Self;

    fn add_entry(&mut self, entry: T);

    fn synchronize(&mut self, other: &mut Self);
}

// https://en.wikipedia.org/wiki/Topological_sorting

// to get nodes after some nodes, traverse and use a set to deduplicate, don't revisit nodes that are already known?
// parents would need to be traversed first, oh no

// starting from the heads to are depth traversal or so and break if you find a remote head?

pub struct DAGHistoryEntry<T> {
    value: T,
    parents: Vec<Rc<DAGHistoryEntry<T>>>,
}

pub struct DAGHistory<T> {
    heads: Vec<DAGHistoryEntry<T>>,
    history: Vec<DAGHistoryEntry<T>>,
}

impl<T> History<T> for DAGHistory<T> {
    fn new() -> Self {
        todo!()
    }

    fn add_entry(&mut self, entry: T) {
        todo!()
    }

    fn synchronize(&mut self, other: &mut Self) {
        todo!()
    }
}
