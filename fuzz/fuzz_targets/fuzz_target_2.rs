#![no_main]
#![feature(get_many_mut)]

use std::{rc::Rc, sync::Once};

use libfuzzer_sys::fuzz_target;
use perfect_collaborative_text_editing::pcte::Pcte;
use tracing::{level_filters::LevelFilter, trace, Level};
use tracing_subscriber::{
    fmt::SubscriberBuilder, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[derive(Debug)]
pub enum FixtureOperation {
    CreateReplica,
    Insert {
        index: usize,
        position: usize,
        character: char,
    },
    Delete {
        index: usize,
        position: usize,
    },
    Synchronize {
        index1: usize,
        index2: usize,
    },
}

#[derive(Debug)]
pub struct FixtureOperations {
    pub operations: Vec<FixtureOperation>,
}

impl<'a> arbitrary::Arbitrary<'a> for FixtureOperations {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len: usize = u.arbitrary_len::<(i32, usize, usize, i32)>()?;

        let mut my_collection = Vec::with_capacity(len);
        let mut replica_count = 0;
        for _ in 0..len {
            let element = if (replica_count == 0) {
                replica_count += 1;
                FixtureOperation::CreateReplica
            } else {
                let max = if replica_count >= 2 { 3 } else { 2 };
                match u.int_in_range(0..=max)? {
                    0 => FixtureOperation::Insert {
                        index: u.int_in_range(0..=replica_count - 1)?,
                        position: u.int_in_range(0..=usize::MAX)?,
                        character: char::from_u32(u.int_in_range(0x21..=0x7e)?).unwrap(),
                    },
                    1 => FixtureOperation::Delete {
                        index: u.int_in_range(0..=replica_count - 1)?,
                        position: u.int_in_range(0..=usize::MAX)?,
                    },
                    2 => {
                        replica_count += 1;
                        FixtureOperation::CreateReplica
                    }
                    3 => {
                        let index1 = u.choose_index(replica_count)?;
                        let mut index2 = u.choose_index(replica_count - 1)?;
                        if index2 == index1 {
                            index2 = replica_count - 1;
                        }
                        FixtureOperation::Synchronize { index1, index2 }
                    }
                    _ => unreachable!(),
                }
            };

            my_collection.push(element);
        }

        Ok(FixtureOperations {
            operations: my_collection,
        })
    }
}

static START: Once = Once::new();

fuzz_target!(|data: FixtureOperations| {
    START.call_once(|| {
        let fmt_layer = tracing_subscriber::fmt::layer();
        tracing_subscriber::registry()
            .with(fmt_layer.with_filter(LevelFilter::TRACE))
            .init();
    });

    trace!("test");

    let mut replicas = Vec::new();
    for operation in data.operations {
        match operation {
            FixtureOperation::CreateReplica => {
                let pcte = Pcte::new(Rc::new(format!("replica{}", replicas.len())));
                replicas.push(pcte);
            }
            FixtureOperation::Synchronize { index1, index2 } => {
                let [replica1, replica2] = replicas.get_many_mut([index1, index2]).unwrap();
                replica1.synchronize(replica2);
                assert_eq!(replica1.text(), replica2.text());
            }
            FixtureOperation::Insert {
                index,
                position,
                character,
            } => {
                let text_length = replicas[index].text().len();
                replicas[index].insert(position % (text_length + 1), character);
            }
            FixtureOperation::Delete { index, position } => {
                let text_length = replicas[index].text().len();
                if text_length != 0 {
                    replicas[index].delete(position % text_length);
                }
            }
        }
    }
});
