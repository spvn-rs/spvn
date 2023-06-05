use bytes::Bytes;
use pyo3::prelude::*;
use pyo3::prelude::{pyclass, pymethods};
use pyo3::Python;
use tracing::{debug};

use pyo3::pyclass::IterNextOutput;

#[pyclass]
pub struct PySyncBodyReceiver {
    pub val: Bytes,
}
#[pymethods]
impl PySyncBodyReceiver {
    /// Ref back to self as an iterator
    fn __call__(slf: PyRef<'_, Self>, py: Python) -> Result<PyObject, PyErr> {
        Ok(slf.val.into_py(py))
    }
}

#[pyclass]
pub struct PyAsyncBodyReceiver {
    pub val: Bytes,
}

#[pymethods]
impl PyAsyncBodyReceiver {
    /// Start the polling loop, ref back to self
    fn __await__(slf: PyRef<'_, Self>) -> Result<PyRef<'_, Self>, PyErr> {
        debug!("await");
        Ok(slf)
    }

    /// Ref back to self as an iterator
    fn __call__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    /// Serialize the poller into a callable iterator for python
    /// Each iteration polls the poller to determine whether to: <br/>
    ///    * a: yield the output <br/>
    ///    * b: raise timeout error <br/>
    ///    * c: poll again (\_\_next\_\_(self))
    fn __next__(slf: PyRef<'_, Self>, py: Python) -> IterNextOutput<PyObject, PyObject> {
        IterNextOutput::Return(slf.val.into_py(py))
    }
}
