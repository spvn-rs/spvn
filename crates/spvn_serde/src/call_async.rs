

use std::time::Duration;
use std::time::Instant;

use crossbeam::channel;

use log::info;

use pyo3::exceptions::*;
use pyo3::prelude::*;
use pyo3::pyclass::IterNextOutput;




/// Implementation of python.futures.Future in rust.
///
///
///
/// A callable which in python acts as the following:
/// ```py
/// def my_scope(async_method_bootstrap: AsyncMethod):
///     fut = async_method_bootstrap() # <- <coroutine object at ...>
///     # print(fut.__await__()) -> iterable(Iter)
///     await fut # -> 0
/// ```
///
/// if [`Poll`] is pending, we send `Self` (signal to loop again), else, we raise
/// stop iteration error, unpack the value into py and signal the call as finished
///
/// Expected call seq
/// 1. \_\___await__\_\_()
/// 2. \_\___call__\_\_()
/// 3. \_\___next__\_\_() <br/>
///     a. Return <br/>
///     b. Yield[`PyFuture`] <br/>
///         * *  \_\___next__\_\_()
#[pyclass]
pub struct PyFuture {
    started: Instant,
    poller_deadline: Duration,
    deadline_task: Duration,
    poll: fn() -> std::task::Poll<bool>,
    val: channel::Receiver<bool>,
}

#[pymethods]
impl PyFuture {
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
    fn __next__(slf: PyRef<'_, Self>) -> IterNextOutput<PyRef<'_, Self>, PyObject> {
        if slf.started.elapsed().as_nanos() > slf.deadline_task.as_nanos() {
            panic!("deadline exceeded",)
        }
        if (slf.poll)().is_pending() {
            info!("pending");
            return pyo3::pyclass::IterNextOutput::Yield(slf);
        }
        let v = match slf.val.recv() {
            Ok(val) => val,
            Err(_e) => unsafe {
                return pyo3::pyclass::IterNextOutput::Return(
                    PyRuntimeError::new_err("Receive failed")
                        .to_object(Python::assume_gil_acquired()),
                );
            },
        };
        unsafe { pyo3::pyclass::IterNextOutput::Return(v.into_py(Python::assume_gil_acquired())) }
    }
}

impl PyFuture {
    pub fn new(poll: fn() -> std::task::Poll<bool>, val: Box<channel::Receiver<bool>>) -> Self {
        let fut = PyFuture {
            poll: poll,
            val: *val,
            started: Instant::now(),
            deadline_task: Duration::from_secs(15),
            poller_deadline: Duration::from_nanos(15),
        };
        fut
    }
}

/// A callable which in python acts as the following:
/// ```py
/// def my_scope(async_method_bootstrap: AsyncMethod):
///     fut = async_method_bootstrap() # <- <coroutine object at ...>
///     # print(fut.__await__()) -> iterable(Iter)
///     await fut # -> 0
/// ```
///
/// if [`Poll`] is pending, we send `Iter(1)` (signal to loop again), else, we increment
/// the inner iterator (which only has len 1) and signal the call as finished
///
///
///
fn abc() {}
// #[pyclass]
// pub struct AsyncMethod {
//     pub poll: fn() -> std::task::Poll<bool>,
//     pub val: Receiver<bool>,
// }

// #[pymethods]
// impl AsyncMethod {
//     fn __call__(&self) -> Result<PyFuture, PyErr> {
//         // let v = self.poll.ta
//         // let v = self.poll.

//         Ok(PyFuture::new(self.poll.clone(), self.val))
//     }
// }
