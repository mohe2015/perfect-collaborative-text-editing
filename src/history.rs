use itertools::Itertools;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::{collections::HashSet, hash::Hasher, rc::Rc};

// TODO FIXME switch to handle based indexing and store elements in parent container

// TODO FIXME implement this with "network" in between because otherwise we just share the data structure itself, which is cheating

// use quic and then two channels, ohne high priority, one low priority
// also support webrtc and webtransport

pub trait History<T> {
    type Item;

    fn new(replica_id: String) -> Self;

    fn add_entry(&mut self, entry: Self::Item);

    fn add_value(&mut self, value: T) -> Self::Item;

    fn new_for_other(&mut self, other: &Self) -> Vec<Self::Item>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VectorClock {
    inner: BTreeMap<String, usize>,
}

impl VectorClock {
    pub fn update_with(&mut self, other: &VectorClock) {
        other
            .inner
            .iter()
            .for_each(|(key, value)| match self.inner.get_mut(key) {
                Some(selfvalue) => *selfvalue = std::cmp::max(*selfvalue, *value),
                None => {
                    self.inner.insert(key.to_owned(), *value);
                }
            })
    }
}

impl PartialOrd for VectorClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let mut result = Ordering::Equal;
        for cmp in self
            .inner
            .iter()
            .merge_join_by(other.inner.iter(), |(a, av), (b, bv)| a.cmp(b))
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
                    result = Ordering::Less;
                }
            }
        }
        Some(result)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct VectorClockHistory<T> {
    pub heads: HashSet<Rc<VectorClockHistoryEntry<T>>>,
    pub history: Vec<Rc<VectorClockHistoryEntry<T>>>,
    pub clock: VectorClock,
    pub replica_id: String,
}

impl<T: Debug> History<T> for VectorClockHistory<T> {
    type Item = Rc<VectorClockHistoryEntry<T>>;

    fn new(replica_id: String) -> Self {
        Self {
            replica_id: replica_id.clone(),
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
        self.clock.update_with(&entry.clock);
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

    #[tracing::instrument(ret)]
    fn new_for_other(&mut self, other: &Self) -> Vec<Self::Item> {
        *(self.clock.inner.get_mut(&self.replica_id).unwrap()) += 1;

        self.history
            .iter()
            .cloned()
            .filter(|elem| {
                other.heads.is_empty() ||
                other.heads.iter().any(|head| {
                    let ret = !(head >= elem);
                    ret
                })
            })
            .collect()
    }
}

// https://en.wikipedia.org/wiki/Topological_sorting

// to get nodes after some nodes, traverse and use a set to deduplicate, don't revisit nodes that are already known?
// parents would need to be traversed first, oh no

// starting from the heads to are depth traversal or so and break if you find a remote head?

// see discussion in computer stuff

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
    fn new_for_other(&mut self, other: &Self) -> Vec<Self::Item> {
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
    use std::sync::Once;

    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _};

    use super::*;

    static START: Once = Once::new();

    fn tracing() {
        START.call_once(|| {
            let fmt_layer = tracing_subscriber::fmt::layer();
            tracing_subscriber::registry()
                .with(fmt_layer.with_filter(LevelFilter::TRACE))
                .init();
        });
    }

    #[test]
    fn vector_clock() {
        tracing();

        let vector_clocka = VectorClock {
            inner: BTreeMap::from_iter([("a".to_owned(), 1)]),
        };
        let vector_clockb = VectorClock {
            inner: BTreeMap::from_iter([("b".to_owned(), 1)]),
        };
        assert!(!(vector_clocka < vector_clockb));
        assert!(!(vector_clocka > vector_clockb));
    }

    #[test]
    fn it_works() {
        let mut history1 = VectorClockHistory::new("a".to_string());
        let a = history1.add_value("a");

        let mut history2 = VectorClockHistory::new("b".to_string());

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2, [a]);

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1, []);

        for entry in new_for_history2 {
            history2.add_entry(entry);
        }

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2, []);

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1, []);
    }

    #[test]
    fn it_works2() {
        let mut history1 = VectorClockHistory::new("a".to_string());
        let a = history1.add_value("a");

        let mut history2 = VectorClockHistory::new("b".to_string());

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2, [a.clone()]);

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1, []);

        for entry in new_for_history2 {
            history2.add_entry(entry);
        }

        assert_eq!(history1.heads, HashSet::from_iter([a.clone()]));
        assert_eq!(history2.heads, HashSet::from_iter([a.clone()]));

        let b = history2.add_value("b");

        assert_eq!(history1.heads, HashSet::from_iter([a.clone()]));
        assert_eq!(history2.heads, HashSet::from_iter([b.clone()]));

        let new_for_history1 = history2.new_for_other(&history1);
        assert_eq!(new_for_history1, [b.clone()]);

        let new_for_history2 = history1.new_for_other(&history2);
        assert_eq!(new_for_history2, []);
    }
}
