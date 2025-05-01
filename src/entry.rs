use std::vec;

use crate::db::{Deserialize, Serialize};

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

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn generate_entry() -> Entry {
        Entry {
            phonetic: "phonetic".to_owned(),
            definition: "definition".to_owned(),
            translation: "translation".to_owned(),
            exchanges: vec!["exchange1".to_owned(), "exchange2".to_owned()],
        }
    }

    #[test]
    fn serialize() {
        let entry = generate_entry();
        let serialized = entry.serialize();
        let deserialized = Entry::deserialize(&serialized);
        assert_eq!(entry.phonetic, deserialized.phonetic);
        assert_eq!(entry.definition, deserialized.definition);
        assert_eq!(entry.translation, deserialized.translation);
        assert_eq!(entry.exchanges, deserialized.exchanges);
    }
}
