use lexi_db::{Deserialize, Serialize};
use std::vec;

#[derive(Default)]
pub struct Entry {
    pub phonetic: String,
    pub definition: String,
    pub translation: String,
    pub exchanges: Vec<String>,
}

impl Serialize for Entry {
    fn serialize(&self) -> Vec<u8> {
        let mut stack = vec![&self.phonetic, &self.definition, &self.translation];
        stack.extend(&self.exchanges);
        stack.serialize()
    }
}

impl Deserialize for Entry {
    fn deserialize(data: &[u8]) -> Self {
        match Vec::<String>::deserialize(data).as_slice() {
            [first, second, third, rest @ ..] => Entry {
                phonetic: first.clone(),
                definition: second.clone(),
                translation: third.clone(),
                exchanges: rest.to_vec(),
            },
            _ => Entry::default(),
        }
    }
}
