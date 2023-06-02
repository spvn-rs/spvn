use pyo3::prelude::*;

trait Call {
    fn call() {}
}

macro_rules! new_ytz {
    ($name:ident, $T: ty, $V: ty) => {{
        use std::time::Duration;
        use std::time::Instant;

        use crossbeam::channel;

        use log::info;

        use pyo3::exceptions::*;
        use pyo3::prelude::*;
        use pyo3::pyclass::IterNextOutput;

        /// Implementation of python.futures.Future in rust.
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
            poll: fn() -> std::task::Poll<$T>,
            val: channel::Receiver<$T>,
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
                let v = match slf.val.recv_timeout(slf.poller_deadline) {
                    Ok(val) => val,
                    Err(_e) => unsafe {
                        return pyo3::pyclass::IterNextOutput::Return(
                            PyRuntimeError::new_err("Receive failed")
                                .to_object(Python::assume_gil_acquired()),
                        );
                    },
                };
                unsafe {
                    pyo3::pyclass::IterNextOutput::Return(v.into_py(Python::assume_gil_acquired()))
                }
            }
        }

        impl PyFuture {
            pub fn new(poll: fn() -> std::task::Poll<$T>, val: Box<channel::Receiver<$T>>) -> Self {
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
    }};
}

#[pyclass]
struct BasicT {}

impl Call for BasicT {
    fn call() {}
}

#[cfg(test)]
mod test {
    use crate::BasicT;
    use pyo3::prelude::*;

    #[test]
    fn test_basic() {
        new_ytz!(BasicT, BasicT, i64);

        let b = BasicT {};

        let p = Python::with_gil(|py| b.into_py(py));
        let a = Python::with_gil(|py| p.getattr(py, "__await__"));

        a.expect("oh no did not work");
    }
}
