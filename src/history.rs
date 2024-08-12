use std::hash::Hash;
use std::{collections::HashSet, hash::Hasher, rc::Rc};

/// Allows hashing a `Rc<T>` value by its address and not its contents.
/// This struct additionally allows cloning and comparing equality
/// by pointer reference.
pub struct RcHashable<T>(pub Rc<T>);

impl<T> Hash for RcHashable<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        Rc::as_ptr(&self.0).hash(state)
    }
}

impl<T> Clone for RcHashable<T> {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}

impl<T> Eq for RcHashable<T> {}

impl<T> PartialEq for RcHashable<T> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl<T> RcHashable<T> {
    pub fn new(value: T) -> Self {
        Self(Rc::new(value))
    }
}

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
    parents: Vec<RcHashable<DAGHistoryEntry<T>>>,
}

pub struct DAGHistory<T> {
    heads: Vec<RcHashable<DAGHistoryEntry<T>>>,
    history: Vec<RcHashable<DAGHistoryEntry<T>>>,
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
        let entry = RcHashable::new(DAGHistoryEntry {
            value: entry,
            parents: heads,
        });
        self.heads.push(entry);
    }

    fn synchronize(&mut self, other: &mut Self) {
        let mut result = Vec::new();
        let mut visited_nodes = HashSet::new();

        for head in &mut self.heads {
            DAGHistory::visit(&mut result, &mut visited_nodes, head)
        }
    }
}

impl<T> DAGHistory<T> {
    fn visit(
        result: &mut Vec<RcHashable<DAGHistoryEntry<T>>>,
        visited_nodes: &mut HashSet<RcHashable<DAGHistoryEntry<T>>>,
        node: &mut RcHashable<DAGHistoryEntry<T>>,
    ) {
        if visited_nodes.contains(node) {
            return;
        }
    }
}
