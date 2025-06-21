use pyo3::prelude::*;
use serialization::*;

#[pyclass]
#[derive(Default)]
pub struct Entry {
    #[pyo3(get, set)]
    pub phonetic: String,
    #[pyo3(get, set)]
    pub definition: String,
    #[pyo3(get, set)]
    pub translation: String,
    #[pyo3(get, set)]
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
    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        Ok(match Vec::<String>::deserialize(r)?.as_slice() {
            [first, second, third, rest @ ..] => Entry {
                phonetic: first.clone(),
                definition: second.clone(),
                translation: third.clone(),
                exchanges: rest.to_vec(),
            },
            _ => Entry::default(),
        })
    }
}
