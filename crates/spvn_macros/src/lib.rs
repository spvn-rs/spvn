use crossbeam::channel;


use pyo3::prelude::*;



use std::time::Duration;








use tokio::task::JoinHandle;

trait IntoPyFuture<T, O, E> {
    fn new(
        poll: JoinHandle<Result<(), E>>,
        val: Box<channel::Receiver<O>>,
        deadline_task: Option<Duration>,
    ) -> T;
}

macro_rules! pyfuture_impl {
    ($name:ident, $Typ: ty,  $Err: ty, $Val: ty) => {{
        use std::time::Duration;
        use std::time::Instant;
        use crossbeam::channel;
        use pyo3::exceptions::*;
        use pyo3::prelude::*;
        use pyo3::pyclass::IterNextOutput;
        use tokio::task::JoinHandle;
        use crate::IntoPyFuture;

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
        pub struct PyFuture   {
            started: Instant,
            deadline_task: Duration,
            poll: JoinHandle<Result<(), $Err>>,
            val: channel::Receiver<$Val>,
        }

        #[pymethods]
        impl PyFuture {
            /// Start the polling loop, ref back to self
            pub fn __await__(slf: PyRef<'_, Self>) -> Result<PyRef<'_, Self>, PyErr> {
                Ok(slf)
            }

            /// Ref back to self as an iterator
            pub fn __call__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
                slf
            }

            /// Serialize the poller into a callable iterator for python
            /// Each iteration polls the poller to determine whether to: <br/>
            ///    * a: yield the output <br/>
            ///    * b: raise timeout error <br/>
            ///    * c: poll again (\_\_next\_\_(self))
            pub fn __next__(slf: PyRef<'_, Self>) -> IterNextOutput<PyRef<'_, Self>, PyObject> {
                if slf.started.elapsed().as_nanos() > slf.deadline_task.as_nanos() {
                    unsafe {
                        return pyo3::pyclass::IterNextOutput::Return(
                            PyRuntimeError::new_err("Context deadline exceeded")
                                .to_object(Python::assume_gil_acquired()),
                        );
                    }
                }
                if !slf.poll.is_finished() {
                    return pyo3::pyclass::IterNextOutput::Yield(slf);
                }
                let v = match slf.val.recv() {
                    Ok(val) => val,
                    Err(_e) => unsafe {
                        // TODO: see err https://github.com/PyO3/pyo3/pull/3202 and https://github.com/PyO3/pyo3/issues/3190 for when this will be fixed that we return an ACUTAL VALUE of the error instead of raising an err
                        return pyo3::pyclass::IterNextOutput::Return(
                            PyRuntimeError::new_err("receive failed")
                                .to_object(Python::assume_gil_acquired()),
                        );
                    },
                };
                unsafe { pyo3::pyclass::IterNextOutput::Return(v.into_py(Python::assume_gil_acquired())) }
            }
        }

        impl IntoPyFuture<PyFuture, $Val, $Err> for $Typ {
            fn new(
                poll: JoinHandle<Result<(), $Err>>,
                val: Box<channel::Receiver<$Val>>,
                deadline_task: Option<Duration>
            ) -> PyFuture {
                let fut = PyFuture {
                    poll: poll,
                    val: *val,
                    started: Instant::now(),
                    deadline_task: deadline_task.unwrap_or(Duration::from_millis(500)),
                };
                fut
            }
        }
    }};
}

#[allow(private_in_public)]
#[pyclass]
struct BasicT {}

#[cfg(test)]
mod test {
    use crate::BasicT;
    use crate::IntoPyFuture;
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_basic() {
        pyfuture_impl!(BasicT, BasicT, Result<BasicT,()>, i64);

        let (tx_bdy, rx_bdy) = crossbeam::channel::bounded::<i64>(1);

        let fut = tokio::task::spawn(async move {
            let re = tx_bdy.send(1);
            re.expect("panic in join handle due to send");
            Ok(())
        });

        Python::with_gil(|p| {
            let _bt = BasicT::new(fut, Box::new(rx_bdy), None);
            let _locals = PyDict::new(p);
            // locals.set_item("b", bt)
        });

        ()
    }
}
