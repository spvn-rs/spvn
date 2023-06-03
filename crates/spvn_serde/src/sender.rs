use crate::{ASGIResponse, ASGIResponsePyDict, InvalidationRationale};
use crossbeam::channel;
use log::info;
use pyo3::{exceptions::PyRuntimeError, prelude::*, pyclass::IterNextOutput, Python};
use std::sync::Arc;

#[pyclass]
pub struct Sender {
    chan: channel::Sender<Arc<ASGIResponse>>,
    sending: bool,
    received: Option<Arc<ASGIResponse>>,
}

impl Sender {
    pub fn new(chan: channel::Sender<Arc<ASGIResponse>>) -> Self {
        Self {
            chan,
            sending: false,
            received: None,
            // mtd: || IterAwait::new(Poll::Ready(true), rx),
        }
    }
}

#[pymethods]
impl Sender {
    // TODO: turn async
    fn __call__<'a>(
        mut slf: PyRefMut<'a, Self>,
        _py: Python<'a>,
        dict: ASGIResponsePyDict,
    ) -> Result<PyRefMut<'a, Self>, InvalidationRationale> {
        if slf.sending {
            return Err(InvalidationRationale {
                message: String::from("did not call await on last send"),
            });
        }

        let res: Result<ASGIResponse, InvalidationRationale> = dict.try_into();
        let received = match res {
            Ok(res) => res,
            Err(e) => {
                #[cfg(debug_assertions)]
                {
                    info!("invalid {:#?}", e.message)
                };
                return Err(e);
            }
        };

        #[cfg(debug_assertions)]
        {
            info!("python sent {:#?}", received)
        };
        slf.received = Some(Arc::new(received));
        slf.sending = true;
        // Ok((slf.mtd)().into_py(py))
        Ok(slf)
    }

    fn __await__(slf: PyRefMut<'_, Self>) -> Result<PyRefMut<'_, Self>, PyErr> {
        let res = slf.chan.send(slf.received.as_ref().unwrap().clone());
        match res {
            Ok(_) => Ok(slf),
            Err(e) => {
                info!("{:#?}", e);
                Err(PyRuntimeError::new_err("an error occured sending the data"))
            }
        }
    }

    fn __next__<'a>(
        mut slf: PyRefMut<'a, Self>,
        py: Python,
    ) -> IterNextOutput<PyRefMut<'a, Self>, PyObject> {
        info!("next");
        slf.sending = false;
        slf.received = None;
        pyo3::pyclass::IterNextOutput::Return(py.None())
    }
}
