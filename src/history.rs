use std::hash::Hash;
use std::{collections::HashSet, hash::Hasher, rc::Rc};

use crate::rc_hashable::RcHashable;

// TODO FIXME switch to handle based indexing and store elements in parent container

pub trait History<T> {
    type Item;

    fn new() -> Self;

    fn add_entry(&mut self, entry: Self::Item);

    fn add_value(&mut self, value: T);

    fn new_for_other(&self, other: &Self) -> Vec<Self::Item>;
}

// https://en.wikipedia.org/wiki/Topological_sorting

// to get nodes after some nodes, traverse and use a set to deduplicate, don't revisit nodes that are already known?
// parents would need to be traversed first, oh no

// starting from the heads to are depth traversal or so and break if you find a remote head?

#[derive(Debug)]
pub struct DAGHistoryEntry<T> {
    pub value: T,
    pub parents: HashSet<RcHashable<DAGHistoryEntry<T>>>,
}

#[derive(Debug)]
pub struct DAGHistory<T> {
    pub heads: HashSet<RcHashable<DAGHistoryEntry<T>>>,
    pub history: Vec<RcHashable<DAGHistoryEntry<T>>>,
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

    fn new_for_other(&self, other: &Self) -> Vec<Self::Item> {
        let mut result = Vec::new();
        let mut visited_nodes = HashSet::new();
        visited_nodes.extend(other.heads.iter().cloned());

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut history1 = DAGHistory::new();
        history1.add_value("a");

        let mut history2 = DAGHistory::new();

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2.len(), 1);

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1.len(), 0);

        for entry in new_for_history2 {
            history2.add_entry(entry);
        }

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2.len(), 0);

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1.len(), 0);
    }
}
