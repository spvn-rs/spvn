use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use bytes_expand::BytesMut;
use colored::Colorize;
use log::info;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
// use cpython::{py_class, PyBytes, PyDict, PyNone, PyResult, Python};
use crate::handlers::tasks::Scheduler;
use futures::lock::Mutex;
use futures::Future;
use http_body::Full;
use hyper::body;
use hyper::{body::Body as IncomingBody, Request, Response};

use spvn_caller::service::caller::Call;
use spvn_caller::service::caller::SyncSafeCaller;
use spvn_cfg::{asgi_from_request, ASGIResponse, ASGIResponsePyDict, InvalidationRationale};
use spvn_serde::ToPy;
use std::collections::HashMap;
use std::marker::Send;
use std::sync::Arc;
use tower_service::Service;
pub enum StateKeys {
    HTTPResponseBody,
    HTTPResponseStart,
}

type State = Arc<Mutex<HashMap<StateKeys, ASGIResponse>>>;
type HeaderState = Arc<Mutex<HashMap<StateKeys, Bytes>>>;
type Sending = Arc<Mutex<BytesMut>>;

type Caller = Arc<Mutex<SyncSafeCaller>>;
type Ra = Result<http::Response<http_body::Full<bytes::Bytes>>, hyper::Error>;

pub struct Bridge {
    state: State,
    caller: Caller,
    send: Sending,
    scheduler: Arc<Scheduler>,
}

impl Bridge {
    pub fn new(caller: Caller, scheduler: Arc<Scheduler>) -> Self {
        Self {
            caller: caller.clone(),
            state: Arc::new(Mutex::new(HashMap::new())),
            send: Arc::new(Mutex::new(BytesMut::new())),
            scheduler: scheduler.clone(),
        }
    }
}

// py_class!(class Receive |py| {
//     data bts: Bytes;

//     def __call__(&self) -> PyResult<PyBytes> {
//         let v: &[u8] = self.bts(py).as_ref();
//         let pb = PyBytes::new(Python::acquire_gil().python(), v);
//         Ok(pb)
//     }
// });

#[pyclass]
struct Sender {
    state: State,
    bytes: Sending,
}

#[pymethods]
impl Sender {
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

fn bail() -> Ra {
    Ok(Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from("Internal Server Error")))
        .unwrap())
}

fn bail_err(err: hyper::Error) -> Ra {
    eprintln!(
        "{} an error occurred in the servicer - {:#?}",
        "error".red(),
        err
    );
    bail()
}

fn bail_py(err: PyErr) -> Ra {
    eprintln!(
        "{} an error occurred in the caller - {:#?}",
        "error".red(),
        err
    );
    bail()
}

impl Service<Request<IncomingBody>> for Pin<Box<Bridge>> {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// calls the function as a service to an incomping request - core asgi implementation
    fn call(&mut self, req: Request<IncomingBody>) -> Self::Future {
        async fn mk_response(
            req: Request<IncomingBody>,
            caller: Caller,
            state: State,
            bytes: Sending,
        ) -> Ra {
            let mut called: Option<Result<(), anyhow::Error>> = None;

            let scope = asgi_from_request(&req)
                .to(Python::acquire_gil().python())
                .to_object(Python::acquire_gil().python());

            // must be called AFTER setting asgi params so we dont steal the ptr
            let body_p: Result<Bytes, hyper::Error> = body::to_bytes(req.into_body()).await;
            let b = match body_p {
                Ok(bts) => bts,
                Err(err) => return bail_err(err),
            };

            // this will allow the python fn to receive the bytes in a lazy manner
            let receiver = Receive {
                // shove it in an arc & contigious piece of mem
                bytes: Arc::new(Mutex::new(b)),
            };

            // this will allow the python fn to send us messages
            let sender = Sender {
                state,
                // clone the ref & incr the ref count
                bytes: bytes.clone(),
            };

            called = Some(
                caller
                    .lock()
                    .await
                    .call(Python::acquire_gil().python(), (scope, receiver, sender)),
            );

            #[cfg(debug_assertions)]
            {
                info!("caller result {:#?}", called)
            }

            match called {
                Some(Ok(_)) => (),
                Some(Err(e)) => {
                    eprintln!("an error occured setting calling the handler - {:#?}", e);
                    return bail();
                }
                None => return bail(),
            }

            let captured = bytes.clone().lock().await.to_vec();
            let b = Full::new(Bytes::from(captured));
            #[cfg(debug_assertions)]
            {
                info!("caller bytes {:#?} ", b)
            }
            Ok(Response::builder().body(b).unwrap())
        }
        let caller = self.caller.clone();
        Box::pin(mk_response(
            req,
            caller,
            self.state.clone(),
            self.send.clone(),
        ))
    }
}
