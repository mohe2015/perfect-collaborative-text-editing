use std::hash::Hash;
use std::{collections::HashSet, hash::Hasher, rc::Rc};

// TODO FIXME switch to handle based indexing and store elements in parent container

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
    type Item;

    fn new() -> Self;

    fn add_entry(&mut self, entry: Self::Item);

    fn add_value(&mut self, value: T);

    fn new_for_other(&mut self, other: &mut Self) -> Vec<Self::Item>;
}

// https://en.wikipedia.org/wiki/Topological_sorting

// to get nodes after some nodes, traverse and use a set to deduplicate, don't revisit nodes that are already known?
// parents would need to be traversed first, oh no

// starting from the heads to are depth traversal or so and break if you find a remote head?

pub struct DAGHistoryEntry<T> {
    value: T,
    parents: HashSet<RcHashable<DAGHistoryEntry<T>>>,
}

pub struct DAGHistory<T> {
    heads: HashSet<RcHashable<DAGHistoryEntry<T>>>,
    history: Vec<RcHashable<DAGHistoryEntry<T>>>,
}

impl<T> History<T> for DAGHistory<T> {
    type Item = RcHashable<DAGHistoryEntry<T>>;

    fn new() -> Self {
        Self {
            heads: HashSet::new(),
            history: Vec::new(),
        }
    }

    fn add_value(&mut self, value: T) {
        let heads = std::mem::take(&mut self.heads);
        let entry = RcHashable::new(DAGHistoryEntry {
            value: value,
            parents: heads,
        });
        self.heads.insert(entry);
    }

    fn add_entry(&mut self, entry: Self::Item) {
        for parent in &entry.0.parents {
            self.heads.remove(parent);
        }
        self.heads.insert(entry.clone());
        self.history.push(entry);
    }

    fn new_for_other(&mut self, other: &mut Self) -> Vec<Self::Item> {
        let mut result = Vec::new();
        let mut visited_nodes = HashSet::new();

        for head in &self.heads {
            DAGHistory::visit(&mut result, &mut visited_nodes, head)
        }

        result.reverse();
        result
    }
}

impl<T> DAGHistory<T> {
    fn visit(
        result: &mut Vec<RcHashable<DAGHistoryEntry<T>>>,
        visited_nodes: &mut HashSet<RcHashable<DAGHistoryEntry<T>>>,
        node: &RcHashable<DAGHistoryEntry<T>>,
    ) {
        if visited_nodes.contains(node) {
            return;
        }

        for parent in &node.0.parents {
            Self::visit(result, visited_nodes, &parent)
        }

        result.push(node.clone());
        visited_nodes.insert(node.clone());
    }
}
