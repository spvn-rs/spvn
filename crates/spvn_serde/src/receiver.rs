use crate::call_async::AsyncMethod;
use crate::state::Polling;
use bytes::Bytes;
use futures::executor;
use futures::lock::Mutex;
use log::info;
use pyo3::prelude::{pyclass, pymethods};
use pyo3::{exceptions::*, prelude::*};
use pyo3::{prelude::*, types::PyBytes, Python};
use std::sync::Arc;

#[pyclass]
/// This class receives the raw bts from the request and feeds into python's interpreter
///
/// Receiver is a PYTHON receiver
/// This differs from RUST receiver, where the relative position is swapped
pub struct Receive {
    pub shot: Polling,
}

#[pymethods]
impl Receive {
    /// Serializes [`Bytes`] from [`Polling`] into [`PyBytes`]
    fn __call__<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        let b = &mut futures::executor::block_on(self.shot.lock());
        #[cfg(debug_assertions)]
        {
            info!("python made call to receive bytes")
        }

        // ** dont call receive_blocking here, it will panic **
        let rcv = executor::block_on(b.recv());
        let bts = match rcv {
            Some(bts) => bts,
            None => return Err(PyIOError::new_err("error receiving caller channel")),
        };

        Ok(PyBytes::new(py, bts.as_ref()))
    }
}

// impl From<Arc<Mutex<Bytes>>> for Receive {
//     fn from(bytes: Arc<Mutex<Bytes>>) -> Self {
//         Self { bytes }
//     }
// }

// #[pymethods]
// impl Receive {
//     fn __call__<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyBytes> {
//         let b = futures::executor::block_on(self.bytes.lock());
//         #[cfg(debug_assertions)]
//         {
//             info!("python made call to receive bytes")
//         }
//         unsafe { Ok(PyBytes::new(py, b.as_ref())) }
//     }
// }
