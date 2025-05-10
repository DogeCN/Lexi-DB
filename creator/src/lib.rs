use db::DBCreator;
use entry::{Entry, PyEntry};
use pyo3::prelude::*;

#[pyclass]
struct PyDBCreator {
    db: DBCreator<Entry>,
}

#[pymethods]
impl PyDBCreator {
    #[new]
    fn new(path: &str, name: &str, name_zh: &str) -> Self {
        PyDBCreator {
            db: DBCreator::new(path, name, name_zh),
        }
    }

    fn insert(&mut self, key: &str, value: &PyEntry) {
        self.db.insert(key, value.to_entry());
    }

    fn export(&self) -> PyResult<()> {
        self.db.export()?;
        Ok(())
    }
}

#[pymodule]
fn creator(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyEntry>()?;
    m.add_class::<PyDBCreator>()?;
    Ok(())
}
