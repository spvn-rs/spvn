use std::sync::atomic::AtomicUsize;

use pyo3::prelude::*;
use pyo3::prelude::{pyclass, pymethods};
use pyo3::Python;
use tracing::debug;
use tracing::log::warn;

use crate::asgi_scope::ASGIEvent;
use colored::Colorize;
use pyo3::pyclass::IterNextOutput;

#[pyclass]
pub struct PySyncEventReceiver {
    pub val: ASGIEvent,
}

#[pymethods]
impl PySyncEventReceiver {
    /// Ref back to self as an iterator
    fn __call__(slf: PyRefMut<'_, Self>, py: Python) -> Result<PyObject, PyErr> {
        Ok(slf.val.clone().to_object(py))
    }
}

#[pyclass]
pub struct PyASyncEventReceiver {
    val: ASGIEvent,
    calls: AtomicUsize,
}

impl PyASyncEventReceiver {
    pub fn new(val: ASGIEvent) -> Self {
        Self {
            val,
            calls: AtomicUsize::new(0),
        }
    }
}

#[pymethods]
impl PyASyncEventReceiver {
    /// Start the polling loop, ref back to self
    fn __await__(slf: PyRefMut<'_, Self>) -> Result<PyRefMut<'_, Self>, PyErr> {
        debug!("await");
        Ok(slf)
    }

    /// Ref back to self as an iterator
    fn __call__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf.calls.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        slf
    }

    /// Serialize the poller into a callable iterator for python
    /// Each iteration polls the poller to determine whether to: <br/>
    ///    * a: yield the output <br/>
    ///    * b: raise timeout error <br/>
    ///    * c: poll again (\_\_next\_\_(self))
    fn __next__(slf: PyRefMut<'_, Self>, py: Python) -> IterNextOutput<PyObject, PyObject> {
        if slf.calls.load(std::sync::atomic::Ordering::Acquire) == 1 {
            return IterNextOutput::Return(slf.val.clone().to_object(py));
        }
        match py.check_signals() {
            Ok(_) => return IterNextOutput::Yield(slf.into_py(py)),
            Err(e) => {
                warn!(
                    "{} {:#?}",
                    "received signal... sending lifecycle termination".red(),
                    e.traceback(py)
                );
            }
        };
        return IterNextOutput::Return(slf.val.clone().to_object(py));
    }
}
