use crate::{
    state::{Sending, State},
    ASGIResponse, ASGIResponsePyDict, InvalidationRationale,
};
use bytes::Bytes;
use futures::lock::Mutex;
use log::info;
use pyo3::{
    prelude::*,
    types::{PyBytes},
    Python,
};
use std::sync::Arc;

#[pyclass]
pub struct Sender {
    pub state: State,
    pub bytes: Sending,
}

#[pymethods]
impl Sender {
    // TODO: turn async
    fn __call__<'a>(
        &self,
        _py: Python<'a>,
        dict: ASGIResponsePyDict,
    ) -> Result<(), InvalidationRationale> {
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
        // let r: Result<&PyAny, PyErr> =
        //     pyo3_asyncio::tokio::future_into_py(py, async move { Ok(()) });
        // let fut = match r {
        //     Ok(fut) => fut,
        //     Err(e) => {
        //         return Err(InvalidationRationale {
        //             message: String::from("couldnt spawn runtime"),
        //         })
        //     }
        // };

        Ok(())
    }
}

#[pyclass]
struct Receive {
    bytes: Arc<Mutex<Bytes>>,
}

#[pymethods]
impl Receive {
    fn __call__<'a>(&mut self, py: Python<'a>) -> PyResult<&'a PyBytes> {
        let b = futures::executor::block_on(self.bytes.lock());
        #[cfg(debug_assertions)]
        {
            info!("python made call to receive bytes")
        }
        unsafe { Ok(PyBytes::new(py, b.as_ref())) }
    }
}
