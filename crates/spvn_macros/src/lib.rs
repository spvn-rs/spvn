





use pyo3::prelude::*;

trait Call {
    fn call() {}
}

macro_rules! new_ytz {
    ($name:ident, $T: ty, $V: ty) => {{
        use log::info;
        use pyo3::prelude::*;

        #[pyclass]
        pub struct Iter {
            poll: Box<std::task::Poll<$V>>,
            inner: std::vec::IntoIter<Option<$V>>,
        }

        #[pymethods]
        impl Iter {
            fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
                #[cfg(debug_assertions)]
                {
                    info!("iter called");
                }
                slf
            }
            fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<Option<$V>> {
                if slf.poll.is_pending() {
                    // let condition: $T = $D;
                    // #[cfg(debug_assertions)]
                    // {
                    //     info!("awaiting python, sending {:}", condition)
                    // }
                    return Some(None);
                }
                let _incr = slf.inner.next();
                // #[cfg(debug_assertions)]
                // {
                //     info!("awaiting python, sending {:#?}", incr)
                // }
                slf.inner.next()
            }
        }

        #[pyclass]
        pub struct IterAwait {
            poll: Box<std::task::Poll<$V>>,
        }

        #[pymethods]
        impl IterAwait {
            fn __await__(slf: PyRef<'_, Self>) -> PyResult<Py<Iter>> {
                Py::new(
                    slf.py(),
                    Iter {
                        poll: slf.poll.clone(),
                        inner: vec![].into_iter(),
                    },
                )
            }
        }

        impl From<std::task::Poll<$V>> for IterAwait {
            fn from(poll: std::task::Poll<$V>) -> Self {
                Self {
                    poll: Box::new(poll),
                }
            }
        }

        #[pymethods]
        impl $name {}
    }};
}

#[pyclass]
struct BasicT {}

impl BasicT {
    fn call() {}
}

#[cfg(test)]
mod test {
    use crate::BasicT;
    use pyo3::prelude::*;

    fn test_basic() {
        new_ytz!(BasicT, BasicT, i64);
    }
}
