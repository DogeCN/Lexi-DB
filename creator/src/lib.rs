use db::DBCreator;
use entry::Entry;
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

    fn insert(&mut self, key: &str, value: &Entry) -> PyResult<()> {
        Ok(self.db.insert(key, value)?)
    }

    fn export(&mut self) -> PyResult<()> {
        Ok(self.db.export()?)
    }
}

#[pymodule]
fn creator(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<Entry>()?;
    m.add_class::<PyDBCreator>()?;
    Ok(())
}
