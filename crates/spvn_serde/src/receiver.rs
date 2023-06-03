use bytes::Bytes;

use crate::call_async::IntoPyFuture;

use log::info;
use pyo3::prelude::*;
use pyo3::prelude::{pyclass, pymethods};
use pyo3::Python;

use pyo3::pyclass::IterNextOutput;

#[pyclass]
pub struct PySyncBodyReceiver {
    pub val: Bytes,
}
#[pymethods]
impl PySyncBodyReceiver {
    // /// Start the polling loop, ref back to self
    // fn __await__(slf: PyRef<'_, Self>) -> Result<PyRef<'_, Self>, PyErr> {
    //     info!("await");
    //     Ok(slf)
    // }

    /// Ref back to self as an iterator
    fn __call__(slf: PyRef<'_, Self>, py: Python) -> Result<PyObject, PyErr> {
        Ok(slf.val.into_py(py))
    }
}

#[pyclass]
pub struct PyAsyncBodyReceiver {
    // started: Instant,
    // deadline_task: Duration,
    // poll: JoinHandle<Result<(), hyper::Error>>,
    // val: channel::Receiver<Bytes>,
    pub val: Bytes,
}

#[pymethods]
impl PyAsyncBodyReceiver {
    /// Start the polling loop, ref back to self
    fn __await__(slf: PyRef<'_, Self>) -> Result<PyRef<'_, Self>, PyErr> {
        info!("await");
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
        // if slf.started.elapsed().as_nanos() > slf.deadline_task.as_nanos() {
        //     unsafe {
        //         return pyo3::pyclass::IterNextOutput::Return(
        //             PyRuntimeError::new_err("Context deadline exceeded")
        //                 .to_object(Python::assume_gil_acquired()),
        //         );
        //     }
        // }
        // if !slf.poll.is_finished() {
        //     return pyo3::pyclass::IterNextOutput::Yield(slf);
        // }
        // let v = match slf.val.recv() {
        //     Ok(val) => val,
        //     Err(_e) => unsafe {
        //         // TODO: see err https://github.com/PyO3/pyo3/pull/3202 and https://github.com/PyO3/pyo3/issues/3190 for when this will be fixed that we return an ACUTAL VALUE of the error instead of raising an err
        //         return pyo3::pyclass::IterNextOutput::Return(
        //             PyRuntimeError::new_err("receive failed")
        //                 .to_object(Python::assume_gil_acquired()),
        //         );
        //     },
        // };
        // unsafe { pyo3::pyclass::IterNextOutput::Return(v.into_py(Python::assume_gil_acquired())) }
        IterNextOutput::Return(slf.val.into_py(py))
    }
}

// impl IntoPyFuture<PyAsyncBodyReceiver, Bytes, hyper::Error> for PyAsyncBodyReceiver {
//     fn new(
//         poll: JoinHandle<Result<(), hyper::Error>>,
//         val: Box<channel::Receiver<Bytes>>,
//         deadline_task: Option<Duration>,
//     ) -> Self {
//         let fut = PyAsyncBodyReceiver {
//             poll: poll,
//             val: *val,
//             started: Instant::now(),
//             deadline_task: deadline_task.unwrap_or(Duration::from_millis(500)),
//         };
//         fut
//     }
// }
