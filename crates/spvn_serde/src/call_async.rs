use log::info;
use pyo3::prelude::*;

#[pyclass]
struct Iter {
    poll: Box<std::task::Poll<bool>>,
    inner: std::vec::IntoIter<usize>,
}

#[pymethods]
impl Iter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<usize> {
        if slf.poll.is_pending() {
            let condition: usize = 1;
            return Some(condition);
        }
        slf.inner.next()
    }
}

// TODO: add value ? the use case doesnt require but it would be cool

/// Rust representation of an awaitable in python
/// Poll returns a boxed task which when [`Poll::Ready`] will raise
/// [`PyStopIteration`]
#[pyclass]
struct IterAwait {
    poll: Box<std::task::Poll<bool>>,
}

#[pymethods]
impl IterAwait {
    fn __await__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
        let s: usize = 1;
        #[cfg(debug_assertions)]
        {
            info!("awaiting python")
        }
        Py::new(
            slf.py(),
            Iter {
                poll: slf.poll.clone(),
                inner: vec![s].into_iter(),
            },
        )
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
#[pyclass]
pub struct AsyncMethod {
    pub poll: std::task::Poll<bool>,
}

#[pymethods]
impl AsyncMethod {
    fn __call__(&self) -> Result<IterAwait, PyErr> {
        Ok(IterAwait {
            poll: Box::new(self.poll.clone()),
        })
    }
}
