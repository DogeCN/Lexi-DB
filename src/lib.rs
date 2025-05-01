mod db;
mod entry;
#[allow(unused_imports)]
use db::{DBCreator, DBReader};
use entry::Entry;
use pyo3::prelude::*;

#[pyclass]
struct PyCreator {
    db: DBCreator<Entry>,
}

#[pymethods]
impl PyCreator {
    #[new]
    fn new(path: String) -> Self {
        let db = DBCreator::new(&path);
        PyCreator { db }
    }

    fn insert(
        &mut self,
        word: String,
        phonetic: String,
        definition: String,
        translation: String,
        exchanges: Vec<String>,
    ) -> PyResult<()> {
        self.db.insert(
            &word,
            Entry {
                phonetic,
                definition,
                translation,
                exchanges,
            },
        );
        Ok(())
    }

    fn export(&mut self) -> PyResult<()> {
        self.db.export()?;
        Ok(())
    }
}

#[pymodule]
fn lexi_db(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyCreator>()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use entry::tests::generate_entry;

    #[test]
    fn test() {
        let path = "tests/test.db";
        let mut db = DBCreator::new(path);
        db.insert("entry1", generate_entry());
        db.insert("entry2", Entry::default());
        db.export().unwrap();
        let mut reader: DBReader<Entry> = DBReader::from(path).unwrap();
        let entry = reader.get("entry1").unwrap();
        let generated = generate_entry();
        assert_eq!(entry.phonetic, generated.phonetic);
        assert_eq!(entry.definition, generated.definition);
        assert_eq!(entry.translation, generated.translation);
        assert_eq!(entry.exchanges, generated.exchanges);
        let entry = reader.get("entry2").unwrap();
        assert_eq!(entry.phonetic, "");
        assert_eq!(entry.definition, "");
        assert_eq!(entry.translation, "");
        assert_eq!(entry.exchanges.len(), 0);
        assert!(reader.get("entry3").is_none());
    }
}
