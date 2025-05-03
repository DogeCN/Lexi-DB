use db::DBReader;
use entry::{Entry, PyEntry};
use pyo3::prelude::*;

#[pyclass]
struct PyDBReader {
    db: DBReader<Entry>,
}

#[pymethods]
impl PyDBReader {
    #[new]
    fn new(path: &str, temp: &str) -> PyResult<Self> {
        Ok(PyDBReader {
            db: DBReader::from(path, temp)?,
        })
    }

    fn __getitem__(&mut self, key: &str) -> Option<PyEntry> {
        self.db.get(key).map(|e| PyEntry::from_entry(&e))
    }

    fn __len__(&self) -> usize {
        self.db.len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.db.contains(key)
    }

    fn keys(&self) -> Vec<&String> {
        self.db.keys()
    }
}

#[pymodule]
fn reader(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyEntry>()?;
    m.add_class::<PyDBReader>()?;
    Ok(())
}
