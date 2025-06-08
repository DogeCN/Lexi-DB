use db::DBReader;
use entry::{Entry, PyEntry};
use matcher::Matcher;
use pyo3::prelude::*;

#[pyclass]
struct PyDBReader {
    db: DBReader<Entry>,
}

#[pymethods]
impl PyDBReader {
    #[new]
    fn new(path: &str, temp: &str) -> PyResult<Self> {
        let db = DBReader::from(path, temp)?;
        Ok(PyDBReader { db })
    }

    fn load(&mut self, py: Python<'_>) -> PyResult<()> {
        Ok(py.allow_threads(|| self.db.load())?)
    }

    #[getter]
    fn name(&self) -> &str {
        self.db.name.as_str()
    }

    #[getter]
    fn name_zh(&self) -> &str {
        self.db.name_zh.as_str()
    }

    fn filter(
        &self,
        py: Python<'_>,
        reader: &mut PyDBReader,
        word: &str,
        seps: Vec<char>,
    ) -> Vec<PyEntry> {
        py.allow_threads(|| {
            let keys = reader.db.filter_keys(word, &seps);
            keys.iter()
                .filter_map(|k| reader.db.get(k).map(|e| e.into()))
                .collect()
        })
    }

    fn __getitem__(&mut self, key: &str) -> Option<PyEntry> {
        self.db.get(key).map(|e| e.into())
    }

    fn __len__(&self) -> usize {
        self.db.len()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.db.contains(key)
    }
}

#[pyclass]
struct PyMatcher {
    matcher: Matcher,
}

#[pymethods]
impl PyMatcher {
    #[new]
    fn new() -> Self {
        PyMatcher {
            matcher: Matcher::new(),
        }
    }

    fn combine(&mut self, py: Python<'_>, reader: &PyDBReader) {
        py.allow_threads(|| self.matcher.add(reader.db.keys()));
    }

    fn find(&self, py: Python<'_>, word: &str) -> Option<&str> {
        py.allow_threads(|| self.matcher.find(word))
    }
}

#[pymodule]
fn reader(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_class::<PyEntry>()?;
    m.add_class::<PyDBReader>()?;
    m.add_class::<PyMatcher>()?;
    Ok(())
}
