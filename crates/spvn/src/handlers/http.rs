use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::Bytes;

use colored::Colorize;
// use crossbeam::channel;
use log::info;
use pyo3::prelude::*;
use spvn_serde::{
    receiver::{PyAsyncBodyReceiver, PySyncBodyReceiver},
    ASGIResponse,
};
use tokio::sync::mpsc::channel;
// use cpython::{py_class, PyBytes, PyDict, PyNone, PyResult, Python};
use crate::handlers::tasks::Scheduler;
use futures::lock::Mutex;
use futures::Future;
use http_body::Full;
use hyper::body;
use hyper::{body::Body as IncomingBody, Request, Response};

use spvn_caller::service::caller::Call;
use spvn_caller::{service::caller::SyncSafeCaller, PyPool};
use spvn_serde::call_async::IntoPyFuture;
use spvn_serde::sender::Sender;
use spvn_serde::state::{Sending, State};
use spvn_serde::{asgi_scope::asgi_from_request, state::Polling};
use std::collections::HashMap;
use std::marker::Send;
use std::sync::Arc;
use tokio::sync::oneshot::channel as oneshot;
use tokio_util::sync::CancellationToken;
use tower_service::Service;

type Caller = Arc<Mutex<SyncSafeCaller>>;
type Ra = Result<http::Response<http_body::Full<bytes::Bytes>>, hyper::Error>;

pub struct Bridge {
    state: State,
    caller: Arc<SyncSafeCaller>,
    send: Sending,
    watch: Polling,
    // ptr only
    scheduler: Arc<Scheduler>,
    cancel: Box<CancellationToken>,
}

impl Bridge {
    pub fn new(caller: Arc<SyncSafeCaller>, scheduler: Arc<Scheduler>) -> Self {
        let (tx, rx) = channel::<Bytes>(3);
        let token = CancellationToken::new();
        Self {
            caller: caller,
            state: Arc::new(Mutex::new(HashMap::new())),
            send: Arc::new(Mutex::new(tx)),
            watch: Arc::new(Mutex::new(rx)),
            scheduler: scheduler.clone(),
            cancel: Box::new(token),
        }
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
            caller: Arc<SyncSafeCaller>,
            _state: State,
            _send: Sending,
            _watch: Polling,
        ) -> Ra {
            let (tx_bdy, rx_bdy) = crossbeam::channel::bounded::<Arc<ASGIResponse>>(4);
            tokio::spawn(async move {
                while let Ok(resp) = rx_bdy.recv() {
                    info!("received at server {:#?}", resp)
                }
            });

            let sender = Sender::new(tx_bdy);

            let (tx_bdy, rx_bdy) = crossbeam::channel::bounded::<Bytes>(1);
            let _bail_super = || return bail();

            // todo: handle
            let _token = CancellationToken::new();
            let asgi = asgi_from_request(&req);
            // let join_body: tokio::task::JoinHandle<_> = tokio::task::spawn(async move {
            //     let body = body::to_bytes(req.into_body()).await;
            //     let _b = match body {
            //         Ok(bts) => bts,
            //         Err(err) => return Err(err),
            //     };
            //     let re = tx_bdy.send(_b);
            //     info!("sending data");
            //     #[cfg(debug_assertions)]
            //     {
            //         info!("response from send {:#?}", re);
            //     }
            //     match re {
            //         Ok(_) => (),
            //         Err(e) => info!("error during call {:#?}", e),
            //     };
            //     Ok(())
            // });
            let body = body::to_bytes(req.into_body()).await;
            let _b = match body {
                Ok(bts) => bts,
                Err(err) => return Err(err),
            };
            // let receiver = PyAsyncBodyReceiver::new(
            //     join_body,
            //     Box::new(rx_bdy),
            //     Some(Duration::from_millis(1500)),
            // );
            let receiver = PyAsyncBodyReceiver { val: _b };
            let join_caller: Result<Result<(), ()>, tokio::task::JoinError> =
                tokio::task::spawn(async move {
                    let res = Python::with_gil(|py| {
                        let obj = asgi.to_object(py);
                        caller.call(
                            py,
                            (
                                obj,
                                // py.None(),
                                // py.None(),
                                receiver,
                                sender,
                                // PyFuture::new(
                                //     join_body,
                                //     Box::new(rx_bdy),
                                //     Some(Duration::from_millis(500)),
                                // ),
                            ),
                        )
                    });
                    info!("{:#?}", res);
                    Ok(())
                })
                .await;

            // let _caller_rr = tokio::join!(join_caller,);
            // let b = Full::new(Bytes::from(captured));
            Ok(Response::builder()
                .body(Full::new(Bytes::from("captured")))
                .unwrap())
        }
        Box::pin(mk_response(
            req,
            self.caller.clone(),
            self.state.clone(),
            self.send.clone(),
            self.watch.clone(),
        ))
    }
}
