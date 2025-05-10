use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use serialization::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug)]
enum LexiError {
    DeserializationError(String),
    SerializationError(String),
}

impl From<LexiError> for PyErr {
    fn from(err: LexiError) -> PyErr {
        match err {
            LexiError::DeserializationError(msg) => {
                PyValueError::new_err(format!("Deserialization Error: {}", msg))
            }
            LexiError::SerializationError(msg) => {
                PyValueError::new_err(format!("Serialization Error: {}", msg))
            }
        }
    }
}

impl From<std::io::Error> for LexiError {
    fn from(err: std::io::Error) -> Self {
        LexiError::DeserializationError(err.to_string())
    }
}

#[pymodule]
fn interface(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(serialize_string, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_uint, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_string_list, m)?)?;
    m.add_function(wrap_pyfunction!(serialize_uint_list, m)?)?;

    m.add_function(wrap_pyfunction!(deserialize_string, m)?)?;
    m.add_function(wrap_pyfunction!(deserialize_uint, m)?)?;
    m.add_function(wrap_pyfunction!(deserialize_string_list, m)?)?;
    m.add_function(wrap_pyfunction!(deserialize_uint_list, m)?)?;

    Ok(())
}

fn serialize<T: Serialize>(py: Python<'_>, value: PyResult<T>) -> Result<PyObject, LexiError> {
    Ok(PyBytes::new(
        py,
        &value
            .map_err(|e: PyErr| LexiError::SerializationError(e.to_string()))?
            .serialize(),
    )
    .into())
}

#[pyfunction]
fn serialize_string(py: Python<'_>, value: &str) -> Result<PyObject, LexiError> {
    serialize(py, Ok(value.to_string()))
}

#[pyfunction]
fn serialize_uint(py: Python<'_>, value: u64) -> Result<PyObject, LexiError> {
    serialize(py, Ok(value))
}

#[pyfunction]
fn serialize_string_list(py: Python<'_>, values: PyObject) -> Result<PyObject, LexiError> {
    serialize(py, values.extract::<Vec<String>>(py))
}

#[pyfunction]
fn serialize_uint_list(py: Python<'_>, values: PyObject) -> Result<PyObject, LexiError> {
    serialize(py, values.extract::<Vec<u64>>(py))
}

fn deserialize<T: Deserialize>(data: &[u8]) -> Result<T, LexiError> {
    T::deserialize(&mut Cursor::new(data))
        .map_err(|e| LexiError::DeserializationError(e.to_string()))
}

#[pyfunction]
fn deserialize_string(data: &[u8]) -> Result<String, LexiError> {
    deserialize(data)
}

#[pyfunction]
fn deserialize_uint(data: &[u8]) -> Result<u64, LexiError> {
    deserialize(data)
}

#[pyfunction]
fn deserialize_string_list(py: Python<'_>, data: &[u8]) -> Result<PyObject, LexiError> {
    #[allow(deprecated)]
    deserialize::<Vec<String>>(data).map(|strings| strings.into_py(py))
}

#[pyfunction]
fn deserialize_uint_list(py: Python<'_>, data: &[u8]) -> Result<PyObject, LexiError> {
    #[allow(deprecated)]
    deserialize::<Vec<u64>>(data).map(|uints| uints.into_py(py))
}
