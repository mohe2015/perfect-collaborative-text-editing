use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::Hash;
use std::{collections::HashSet, hash::Hasher, rc::Rc};

use itertools::Itertools;

// TODO FIXME switch to handle based indexing and store elements in parent container

// TODO FIXME implement this with "network" in between because otherwise we just share the data structure itself, which is cheating

// use quic and then two channels, ohne high priority, one low priority
// also support webrtc and webtransport

pub trait History<T> {
    type Item;

    fn new(replica_id: String) -> Self;

    fn add_entry(&mut self, entry: Self::Item);

    fn add_value(&mut self, value: T) -> Self::Item;

    fn new_for_other(&self, other: &Self) -> Vec<Self::Item>;
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct VectorClock {
    inner: BTreeMap<String, usize>,
}

impl PartialOrd for VectorClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut result = Ordering::Equal;
        for cmp in self
            .inner
            .iter()
            .merge_join_by(other.inner.iter(), |a, b| a.cmp(b))
        {
            match cmp {
                itertools::EitherOrBoth::Both((a, ac), (b, bc)) => match ac.cmp(bc) {
                    Ordering::Less => {
                        if result == Ordering::Greater {
                            return None;
                        }
                        result = Ordering::Less;
                    }
                    Ordering::Equal => {}
                    Ordering::Greater => {
                        if result == Ordering::Less {
                            return None;
                        }
                        result = Ordering::Greater;
                    }
                },
                itertools::EitherOrBoth::Left(_) => {
                    if result == Ordering::Less {
                        return None;
                    }
                    result = Ordering::Greater;
                }
                itertools::EitherOrBoth::Right(_) => {
                    if result == Ordering::Greater {
                        return None;
                    }
                    result == Ordering::Less;
                }
            }
        }
        Some(result)
    }
}

pub struct VectorClockHistoryEntry<T> {
    pub value: T,
    pub clock: VectorClock,
}

impl<T> PartialEq for VectorClockHistoryEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.clock == other.clock
    }
}

impl<T> PartialOrd for VectorClockHistoryEntry<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.clock.partial_cmp(&other.clock)
    }
}

impl<T> Eq for VectorClockHistoryEntry<T> {}

impl<T> Hash for VectorClockHistoryEntry<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.clock.hash(state);
    }
}

pub struct VectorClockHistory<T> {
    pub heads: HashSet<Rc<VectorClockHistoryEntry<T>>>,
    pub history: Vec<Rc<VectorClockHistoryEntry<T>>>,
    pub clock: VectorClock,
}

impl<T: Eq + Hash> History<T> for VectorClockHistory<T> {
    type Item = Rc<VectorClockHistoryEntry<T>>;

    fn new(replica_id: String) -> Self {
        Self {
            heads: HashSet::new(),
            history: Vec::new(),
            clock: VectorClock {
                inner: BTreeMap::from_iter([(replica_id, 1)]),
            },
        }
    }

    fn add_entry(&mut self, entry: Self::Item) {
        self.history.push(entry.clone());
        self.heads.retain(|elem| !(elem < &entry));
        self.heads.insert(entry);
    }

    fn add_value(&mut self, value: T) -> Self::Item {
        let entry = Rc::new(VectorClockHistoryEntry {
            value,
            clock: self.clock.clone(),
        });
        self.history.push(entry.clone());
        self.heads.insert(entry.clone());
        entry
    }

    fn new_for_other(&self, other: &Self) -> Vec<Self::Item> {
        todo!()
    }
}

// https://en.wikipedia.org/wiki/Topological_sorting

// to get nodes after some nodes, traverse and use a set to deduplicate, don't revisit nodes that are already known?
// parents would need to be traversed first, oh no

// starting from the heads to are depth traversal or so and break if you find a remote head?

#[derive(Debug, PartialOrd, Ord)]
pub struct DAGHistoryEntry<T> {
    pub value: T,
    pub parents: BTreeSet<Rc<DAGHistoryEntry<T>>>,
}

impl<T: PartialEq> PartialEq for DAGHistoryEntry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.parents == other.parents
    }
}

impl<T: PartialEq> Eq for DAGHistoryEntry<T> {}

#[derive(Debug)]
pub struct DAGHistory<T> {
    pub heads: BTreeSet<Rc<DAGHistoryEntry<T>>>,
    pub history: Vec<Rc<DAGHistoryEntry<T>>>,
}

// TODO fuzz this

impl<T: Ord + std::fmt::Debug> History<T> for DAGHistory<T> {
    type Item = Rc<DAGHistoryEntry<T>>;

    fn new(replica_id: String) -> Self {
        Self {
            heads: BTreeSet::new(),
            history: Vec::new(),
        }
    }

    fn add_value(&mut self, value: T) -> Self::Item {
        let heads = std::mem::take(&mut self.heads);
        let entry = Rc::new(DAGHistoryEntry {
            value: value,
            parents: heads,
        });
        self.heads.insert(entry.clone());
        entry
    }

    // TODO FIXME don't get the others object but create your own?
    fn add_entry(&mut self, entry: Self::Item) {
        for parent in &entry.parents {
            self.heads.remove(parent);
        }
        self.heads.insert(entry.clone());
        self.history.push(entry);
    }

    #[tracing::instrument(ret)]
    fn new_for_other(&self, other: &Self) -> Vec<Self::Item> {
        let mut result = Vec::new();
        let mut visited_nodes = BTreeSet::new();
        visited_nodes.extend(other.heads.iter().cloned());

        for head in &self.heads {
            DAGHistory::visit(&mut result, &mut visited_nodes, head)
        }

        result
    }
}

impl<T: Ord> DAGHistory<T> {
    fn visit(
        result: &mut Vec<Rc<DAGHistoryEntry<T>>>,
        visited_nodes: &mut BTreeSet<Rc<DAGHistoryEntry<T>>>,
        node: &Rc<DAGHistoryEntry<T>>,
    ) {
        if visited_nodes.contains(node) {
            return;
        }

        for parent in &node.parents {
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
        let mut history1 = DAGHistory::new("a".to_string());
        history1.add_value("a");

        let mut history2 = DAGHistory::new("b".to_string());

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

    #[test]
    fn it_works2() {
        let mut history1 = DAGHistory::new("a".to_string());
        let a = history1.add_value("a");

        let mut history2 = DAGHistory::new("b".to_string());

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2, Vec::from_iter([a.clone()]));

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1, Vec::from_iter([]));

        for entry in new_for_history2 {
            history2.add_entry(entry);
        }

        assert_eq!(history1.heads, BTreeSet::from_iter([a.clone()]));
        assert_eq!(history2.heads, BTreeSet::from_iter([a.clone()]));

        let b = history2.add_value("b");

        assert_eq!(history1.heads, BTreeSet::from_iter([a.clone()]));
        assert_eq!(history2.heads, BTreeSet::from_iter([b.clone()]));

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1, Vec::from_iter([b.clone()]));

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2, Vec::from_iter([])); // a
    }
}
