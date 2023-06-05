use crate::{ASGIResponse, AsgiDict, InvalidationRationale};
use crossbeam::channel;
use pyo3::{
    exceptions::PyRuntimeError, prelude::*, pyclass::IterNextOutput, types::PyDict, Python,
};
use tracing::{debug, log::warn};

#[pyclass]
pub struct Sender {
    chan: channel::Sender<ASGIResponse>,
    sending: bool,
    received: Option<ASGIResponse>,
}

impl Sender {
    pub fn new(chan: channel::Sender<ASGIResponse>) -> Self {
        Self {
            chan,
            sending: false,
            received: None,
        }
    }
}

#[pymethods]
impl Sender {
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
        let res: Result<ASGIResponse, InvalidationRationale> = ad.try_into();
        let received = match res {
            Ok(res) => res,
            Err(e) => {
                #[cfg(debug_assertions)]
                {
                    warn!("invalid {:#?}", e.message)
                };
                return Err(e);
            }
        };

        #[cfg(debug_assertions)]
        {
            debug!("python sent {:#?}", received)
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
                    debug!("{:#?}", e);
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
        debug!("next");
        slf.sending = false;
        slf.received = None;
        pyo3::pyclass::IterNextOutput::Return(py.None())
    }
}
