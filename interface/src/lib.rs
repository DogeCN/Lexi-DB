use lz4::{Decoder, EncoderBuilder};
use pyo3::{exceptions::PyValueError, prelude::*, types::PyBytes};
use serialization::{Deserialize, Serialize};
use std::io::{Cursor, Read, Write};

enum Error {
    DeserializationError(String),
    SerializationError(String),
}

impl From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        match err {
            Error::DeserializationError(msg) => {
                PyValueError::new_err(format!("Deserialization Error: {}", msg))
            }
            Error::SerializationError(msg) => {
                PyValueError::new_err(format!("Serialization Error: {}", msg))
            }
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::DeserializationError(err.to_string())
    }
}

#[pymodule]
fn interface(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Serializer>()?;
    m.add_class::<Deserializer>()?;
    m.add_class::<Compressor>()?;

    Ok(())
}

fn serialize<T: Serialize>(py: Python<'_>, value: PyResult<T>) -> PyResult<PyObject> {
    Ok(PyBytes::new(
        py,
        &value
            .map_err(|e: PyErr| Error::SerializationError(e.to_string()))?
            .serialize(),
    )
    .into())
}

fn deserialize<T: Deserialize>(data: &[u8]) -> PyResult<T> {
    Ok(T::deserialize(&mut Cursor::new(data))
        .map_err(|e| Error::DeserializationError(e.to_string()))?)
}

#[pyclass]
struct Serializer;

#[pymethods]
impl Serializer {
    #[staticmethod]
    fn from_string(py: Python<'_>, value: &str) -> PyResult<PyObject> {
        serialize(py, Ok(value.to_string()))
    }

    #[staticmethod]
    fn from_uint(py: Python<'_>, value: u64) -> PyResult<PyObject> {
        serialize(py, Ok(value))
    }

    #[staticmethod]
    fn from_string_list(py: Python<'_>, values: PyObject) -> PyResult<PyObject> {
        serialize(py, values.extract::<Vec<String>>(py))
    }

    #[staticmethod]
    fn from_uint_list(py: Python<'_>, values: PyObject) -> PyResult<PyObject> {
        serialize(py, values.extract::<Vec<u64>>(py))
    }
}

#[pyclass]
struct Deserializer;

#[pymethods]
impl Deserializer {
    #[staticmethod]
    fn to_string(data: &[u8]) -> PyResult<String> {
        deserialize(data)
    }

    #[staticmethod]
    fn to_uint(data: &[u8]) -> PyResult<u64> {
        deserialize(data)
    }

    #[staticmethod]
    fn to_string_list(py: Python<'_>, data: &[u8]) -> PyResult<PyObject> {
        #[allow(deprecated)]
        deserialize::<Vec<String>>(data).map(|strings| strings.into_py(py))
    }

    #[staticmethod]
    fn to_uint_list(py: Python<'_>, data: &[u8]) -> PyResult<PyObject> {
        #[allow(deprecated)]
        deserialize::<Vec<u64>>(data).map(|uints| uints.into_py(py))
    }
}

#[pyclass]
struct Compressor;

#[pymethods]
impl Compressor {
    #[staticmethod]
    fn compress(py: Python<'_>, data: &[u8]) -> PyResult<PyObject> {
        let mut encoder = EncoderBuilder::new().level(4).build(Vec::new())?;
        encoder.write_all(data)?;
        let (data, _) = encoder.finish();
        Ok(PyBytes::new(py, &data).into())
    }

    #[staticmethod]
    fn decompress(py: Python<'_>, data: &[u8]) -> PyResult<PyObject> {
        let mut decoder = Decoder::new(Cursor::new(data))?;
        let mut data = Vec::new();
        decoder.read_to_end(&mut data)?;
        Ok(PyBytes::new(py, &data).into())
    }
}
