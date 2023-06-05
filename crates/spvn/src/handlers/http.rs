use std::{
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use colored::Colorize;
use http::response::Builder;
use pyo3::prelude::*;
use spvn_serde::{body_receiver::PyAsyncBodyReceiver, coalesced, state::StateMap, ASGIResponse};
use tracing::{debug};

use crate::handlers::tasks::Scheduler;

use futures::lock::Mutex;
use futures::Future;
use http_body::Full;
use hyper::body;
use hyper::{body::Body as IncomingBody, Request, Response};
use spvn_caller::service::caller::Call;
use spvn_caller::service::caller::SyncSafeCaller;
use spvn_serde::asgi_scope::asgi_from_request;
use spvn_serde::asgi_sender::Sender;
use spvn_serde::state::State;

use std::marker::Send;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tower_service::Service;
type Caller = Arc<Mutex<SyncSafeCaller>>;
type Ra = Result<http::Response<http_body::Full<bytes::Bytes>>, hyper::Error>;

pub struct Bridge {
    // state: State,
    caller: Arc<SyncSafeCaller>,
    // ptr only
    scheduler: Arc<Scheduler>,
    cancel: Box<CancellationToken>,
    peer: SocketAddr,
    server: SocketAddr,
}

impl Bridge {
    pub fn new(
        caller: Arc<SyncSafeCaller>,
        scheduler: Arc<Scheduler>,
        peer: SocketAddr,
        server: SocketAddr,
    ) -> Self {
        let token = CancellationToken::new();
        Self {
            caller: caller,
            // state: Arc::new(Mutex::new(HashMap::new())),
            scheduler: scheduler.clone(),
            cancel: Box::new(token),
            peer,
            server,
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

struct SendResponse<'a>(&'a std::sync::Mutex<Option<(Builder, Bytes)>>);

impl<'a> SendResponse<'a> {
    fn replace(&mut self, other: Option<(Builder, Bytes)>) {
        let mut state = self.0.lock().unwrap();
        *state = other;
    }
}

impl Service<Request<IncomingBody>> for Bridge {
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
            state: State,
            server: SocketAddr,
            peer: SocketAddr,
        ) -> Ra {
            // let final =
            let (tx_bdy, rx_bdy) = crossbeam::channel::bounded::<ASGIResponse>(4);
            let (tx_builder, rx_builder) = crossbeam::channel::bounded::<(Builder, Bytes)>(1);
            let _resp_mu: Arc<std::sync::Mutex<Option<(Builder, Bytes)>>> =
                Arc::new(std::sync::Mutex::new(None));
            // let mut sr = SendResponse(&*resp_mu);

            tokio::spawn(async move {
                while let Ok(resp) = rx_bdy.recv() {
                    let mut state = state.lock().await;
                    state.0.insert(resp);
                }
                let state = state.lock().await;
                let response = coalesced::coslesce_from_state(&state, Response::builder(), true);
                // sr.replace(Some(response));

                // captured = Some("".to_string());
                let _res = tx_builder.send(response);

                // match res {
                //     Ok(_r) => (),
                //     Err(_e) => panic!("couldnt send response to channel"),
                // }
            });
            let sender = Sender::new(tx_bdy);

            let _bail_super = || return bail();

            // todo: handle
            let _token = CancellationToken::new();
            let asgi = asgi_from_request(&req, server, peer);

            let body = body::to_bytes(req.into_body()).await;
            let _b = match body {
                Ok(bts) => bts,
                Err(err) => return bail_err(err),
            };
            let receiver = PyAsyncBodyReceiver { val: _b };

            let join_caller: Result<Result<(), ()>, tokio::task::JoinError> =
                tokio::task::spawn(async move {
                    let res = Python::with_gil(|py| {
                        let obj = asgi.to_object(py);
                        caller.call(py, (obj, receiver, sender))
                    });
                    debug!("{:#?}", res);
                    Ok(())
                })
                .await;

            match join_caller {
                Ok(call) => match call {
                    Ok(_) => (),
                    Err(_pye) => {
                        // eprintln!("{:#?}", pye);
                        return bail();
                    }
                },
                Err(pye) => {
                    eprintln!("{}", pye)
                }
            }

            match rx_builder.recv() {
                Ok((builder, bts)) => return Ok(builder.body(Full::new(bts)).unwrap()),
                Err(_) => return bail(),
            }
            // match (*resp_mu).lock().unwrap().take() {
            //     Some((builder, body)) => {
            //         let resp = builder.body(Full::new(body)).unwrap();

            //         // #[cfg(debug_assertions)]
            //         // {
            //         //     println!("{:#?}", resp)
            //         // }
            //         return Ok(resp);
            //     }
            //     None => {
            //         #[cfg(debug_assertions)]
            //         {
            //             eprintln!("the receiver failed")
            //         }
            //         return bail();
            //     }
            // }
        }

        let hm: StateMap = StateMap::default();
        let state: State = Arc::new(Mutex::new(hm));
        Box::pin(mk_response(
            req,
            self.caller.clone(),
            state,
            self.server,
            self.peer,
        ))
    }
}
