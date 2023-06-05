use bytes;
use pyo3::{prelude::*, types::PyBytes};

#[pyclass]
struct AsgiResponse {
    _type: String,
    body: Option<bytes::Bytes>,
    headers: Vec<(String, Vec<u8>)>,
}

#[pyfunction]
fn new_asgi_response(
    _type: String,
    body: Option<&PyBytes>,
    headers: Vec<(String, Vec<u8>)>,
) -> AsgiResponse {
    let _bts = body.unwrap().as_bytes();
    // let body = bytes::Bytes::from(bts);

    AsgiResponse {
        _type,
        body: None,
        headers,
    }
}

#[pymodule]
fn spvn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    Python::with_gil(|py| {
        assert!(py.version_info() >= (3, 10));
    });

    m.add_function(wrap_pyfunction!(new_asgi_response, m)?)?;
    m.add_class::<AsgiResponse>()?;

    Ok(())
}
