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
    heads: Vec<Rc<DAGHistoryEntry<T>>>,
    history: Vec<Rc<DAGHistoryEntry<T>>>,
}

impl<T> History<T> for DAGHistory<T> {
    fn new() -> Self {
        Self {
            heads: Vec::new(),
            history: Vec::new(),
        }
    }

    fn add_entry(&mut self, entry: T) {
        let heads = std::mem::take(&mut self.heads);
        let entry = Rc::new(DAGHistoryEntry {
            value: entry,
            parents: heads,
        });
        self.heads.push(entry);
    }

    fn synchronize(&mut self, other: &mut Self) {
        todo!()
    }
}
