#![no_main]

use libfuzzer_sys::fuzz_target;
use perfect_collaborative_text_editing::pcte::Pcte;

#[derive(Debug)]
pub enum FixtureOperation {
    Insert { position: usize, character: char },
    Delete { position: usize },
}

#[derive(Debug)]
pub struct FixtureOperations {
    pub operations: Vec<FixtureOperation>,
}

impl<'a> arbitrary::Arbitrary<'a> for FixtureOperations {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let len = u.arbitrary_len::<(bool, u8, u8)>()?;

        let mut my_collection = Vec::with_capacity(len);
        let mut size = 0;
        for _ in 0..len {
            let element = if size == 0 {
                let element = FixtureOperation::Insert {
                    position: u.int_in_range(0..=size)?,
                    character: char::from_u32(u.int_in_range(0x21..=0x7e)?).unwrap(),
                };
                size += 1;
                element
            } else {
                match bool::arbitrary(u)? {
                    true => {
                        let element = FixtureOperation::Insert {
                            position: u.int_in_range(0..=size)?,
                            character: char::from_u32(u.int_in_range(0x21..=0x7e)?).unwrap(),
                        };
                        size += 1;
                        element
                    }
                    false => {
                        let element = FixtureOperation::Delete {
                            position: u.int_in_range(0..=size - 1)?,
                        };
                        size -= 1;
                        element
                    }
                }
            };

            my_collection.push(element);
        }

        Ok(FixtureOperations {
            operations: my_collection,
        })
    }
}

fuzz_target!(|data: FixtureOperations| {
    let mut pcte = Pcte::new();
    let mut string = String::new();
    for operation in data.operations {
        match operation {
            FixtureOperation::Insert {
                position,
                character,
            } => {
                pcte.insert(position, character);
                let byte_index = if position == 0 {
                    0
                } else {
                    let (a, b) = string.char_indices().nth(position - 1).unwrap();
                    a + b.len_utf8()
                };
                string.insert(byte_index, character);
                assert_eq!(pcte.text(), string);
            }
            FixtureOperation::Delete { position } => {
                pcte.delete(position);
                let byte_index = if position == 0 {
                    0
                } else {
                    let (a, b) = string.char_indices().nth(position - 1).unwrap();
                    a + b.len_utf8()
                };
                string.remove(byte_index);
                assert_eq!(pcte.text(), string);
            }
        }
    }
});
