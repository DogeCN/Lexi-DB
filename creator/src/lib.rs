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
    fn new(path: &str, name: &str, name_zh: &str) -> PyResult<Self> {
        Ok(PyDBCreator {
            db: DBCreator::new(path, name, name_zh)?,
        })
    }

    fn insert(&mut self, key: &str, value: &PyEntry) -> PyResult<()> {
        Ok(self.db.insert(key, value.to_entry())?)
    }

    fn export(&mut self) -> PyResult<()> {
        Ok(self.db.export()?)
    }
}

#[pymodule]
fn creator(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyEntry>()?;
    m.add_class::<PyDBCreator>()?;
    Ok(())
}
