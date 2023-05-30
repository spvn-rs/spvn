
use bytes::Bytes;
use futures::lock::Mutex;
use log::info;
use pyo3::prelude::{pyclass, pymethods};
use pyo3::{
    prelude::*,
    types::{PyBytes},
    Python,
};
use std::sync::Arc;

#[pyclass]
pub struct Receive {
    pub bytes: Arc<Mutex<Bytes>>,
}

impl From<Arc<Mutex<Bytes>>> for Receive {
    fn from(bytes: Arc<Mutex<Bytes>>) -> Self {
        Self { bytes }
    }
}

#[pymethods]
impl Receive {
    fn __call__<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        let b = futures::executor::block_on(self.bytes.lock());
        #[cfg(debug_assertions)]
        {
            info!("python made call to receive bytes")
        }
        unsafe { Ok(PyBytes::new(py, b.as_ref())) }
    }
}
