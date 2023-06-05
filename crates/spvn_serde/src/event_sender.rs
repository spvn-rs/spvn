use std::sync::Arc;

use crate::{asgi_scope::ASGIEvent, ASGIResponse, AsgiDict, InvalidationRationale};
use crossbeam::channel;
use log::info;
use pyo3::{
    exceptions::PyRuntimeError, prelude::*, pyclass::IterNextOutput, types::PyDict, Python,
};

#[pyclass]
pub struct EventSender {
    chan: channel::Sender<ASGIEvent>,
    sending: bool,
    received: Option<ASGIEvent>,
}

impl EventSender {
    pub fn new(chan: channel::Sender<ASGIEvent>) -> Self {
        Self {
            chan,
            sending: false,
            received: None,
        }
    }
}

#[pymethods]
impl EventSender {
    fn __call__<'a>(
        mut slf: PyRefMut<'a, Self>,
        _py: Python<'a>,
        dict: &'a PyDict,
    ) -> Result<PyRefMut<'a, Self>, InvalidationRationale> {
        if slf.sending {
            return Err(InvalidationRationale {
                message: String::from("did not call await on last send"),
            });
        }
        let ad = AsgiDict(dict);
        let res: Result<ASGIEvent, InvalidationRationale> = ad.try_into();
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
        slf.received = Some(received);
        slf.sending = true;
        Ok(slf)
    }

    fn __await__(slf: PyRefMut<'_, Self>) -> Result<PyRefMut<'_, Self>, PyErr> {
        let r = slf.received.as_ref();
        if r.is_some() {
            let res = slf.chan.send(r.unwrap().to_owned());
            match res {
                Ok(_) => Ok(slf),
                Err(e) => {
                    info!("{:#?}", e);
                    Err(PyRuntimeError::new_err("an error occured sending the data"))
                }
            }
        } else {
            Err(PyRuntimeError::new_err("await before sending data"))
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
