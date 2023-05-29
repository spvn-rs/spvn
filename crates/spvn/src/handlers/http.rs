use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use colored::Colorize;
// use cpython::{prepare_freethreaded_python, _detail::ffi::PyObject};
use pyo3::{prelude::*, types::IntoPyDict};

// use cpython::{py_class, PyBytes, PyDict, PyNone, PyResult, Python};
use futures::lock::Mutex;
use futures::Future;
use http_body::Full;
use hyper::body;
use hyper::{body::Body as IncomingBody, Request, Response};
use pyo3::types::PyDict;
use spvn_caller::service::caller::Call;
use spvn_caller::service::caller::SyncSafeCaller;
use spvn_cfg::{asgi_from_request, ASGIResponse, ASGIType};
use spvn_serde::ToPy;
use std::collections::HashMap;
use std::marker::Send;
use std::sync::Arc;
use tower_service::Service;

pub enum StateKeys {
    HTTPResponseBody,
    HTTPResponseStart,
    ResponseBody,
}

type State = Arc<Mutex<HashMap<StateKeys, Bytes>>>;
type HeaderState = Arc<Mutex<HashMap<StateKeys, Bytes>>>;

type Caller = Arc<Mutex<SyncSafeCaller>>;
type Ra = Result<http::Response<http_body::Full<bytes::Bytes>>, hyper::Error>;

pub struct Bridge {
    pub state: State,
    pub caller: Caller,
}

// py_class!(class Receive |py| {
//     data bts: Bytes;

//     def __call__(&self) -> PyResult<PyBytes> {
//         let v: &[u8] = self.bts(py).as_ref();
//         let pb = PyBytes::new(Python::acquire_gil().python(), v);
//         Ok(pb)
//     }
// });

// py_class!(class Sender |py| {
//     data state: State;

//     def __call__(&self, dict: PyDict) -> PyResult<PyNone> {

//         Ok(PyNone)
//     }
// });

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
        async fn mk_response(req: Request<IncomingBody>, caller: Caller) -> Ra {
            let mut called: Option<Result<(), anyhow::Error>> = None;

            let scope = asgi_from_request(&req);
            {}
            let s = scope
                .to(Python::acquire_gil().python())
                .to_object(Python::acquire_gil().python());

            // must be called AFTER setting asgi params so we dont steal the ptr
            let body_p = body::to_bytes(req.into_body()).await;
            let b = match body_p {
                Ok(bts) => bts,
                Err(err) => return bail_err(err),
            };
            called = Some(
                caller
                    .lock()
                    .await
                    .call(Python::acquire_gil().python(), (s, 1, 2)),
            );
            // let fu = Receive::create_instance(Python::acquire_gil().python(), b);
            // let receive = match fu {
            //     Ok(receive) => receive,
            //     Err(err) => return bail_py(err),
            // };

            // base.set_item( "receive", receive);

            match called {
                Some(Ok(_)) => (),
                Some(Err(e)) => {
                    eprintln!("an error occured setting calling the handler - {:#?}", e);
                    return bail();
                }
                None => return bail(),
            }

            Ok(Response::builder()
                .body(Full::new(Bytes::from("tada")))
                .unwrap())
        }
        let caller = self.caller.clone();
        Box::pin(mk_response(req, caller))
    }
}
