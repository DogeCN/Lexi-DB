mod entry;
pub use entry::Entry;
use pyo3::prelude::*;

#[pyclass]
pub struct PyEntry {
    #[pyo3(get, set)]
    pub phonetic: String,
    #[pyo3(get, set)]
    pub definition: String,
    #[pyo3(get, set)]
    pub translation: String,
    #[pyo3(get, set)]
    pub exchanges: Vec<String>,
}

#[pymethods]
impl PyEntry {
    #[new]
    pub fn new(
        phonetic: &str,
        definition: &str,
        translation: &str,
        exchanges: Vec<String>,
    ) -> Self {
        PyEntry {
            phonetic: phonetic.to_owned(),
            definition: definition.to_owned(),
            translation: translation.to_owned(),
            exchanges,
        }
    }
}

impl From<Entry> for PyEntry {
    fn from(entry: Entry) -> Self {
        PyEntry {
            phonetic: entry.phonetic,
            definition: entry.definition,
            translation: entry.translation,
            exchanges: entry.exchanges,
        }
    }
}

impl From<&PyEntry> for Entry {
    fn from(entry: &PyEntry) -> Self {
        {
            let entry: &PyEntry = &entry;
            Entry {
                phonetic: entry.phonetic.clone(),
                definition: entry.definition.clone(),
                translation: entry.translation.clone(),
                exchanges: entry.exchanges.clone(),
            }
        }
    }
}
