

use crate::{
    state::{Sending, State},
    ASGIResponse, ASGIResponsePyDict, InvalidationRationale,
};



use log::info;
use pyo3::{prelude::*, Python};
use std::sync::Arc;

use tokio::sync::oneshot::Receiver;

#[pyclass]
pub struct Sender {
    pub state: State,
    pub bytes: Sending,
    // pub mtd: fn() -> IterAwait,
}

impl Sender {
    pub fn new(bytes: Sending, state: State, _rx: Arc<Receiver<bool>>) -> Self {
        Self {
            state,
            bytes,
            // mtd: || IterAwait::new(Poll::Ready(true), rx),
        }
    }
}

#[pymethods]
impl Sender {
    // TODO: turn async
    fn __call__<'a>(
        _slf: PyRef<'_, Self>,
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
        // Ok((slf.mtd)().into_py(py))
        Ok(())
    }
}
