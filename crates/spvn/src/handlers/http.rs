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
use tokio::task::JoinError;
use tracing::{debug, error};

use crate::handlers::tasks::Scheduler;

use futures::lock::Mutex;
use futures::Future;
use http_body::Full;
use hyper::body;
use hyper::{body::Body as IncomingBody, Request, Response};
use spvn_caller::service::caller::{Call, Caller};
use spvn_serde::asgi_scope::asgi_from_request;
use spvn_serde::asgi_sender::Sender;
use spvn_serde::state::State;

use std::marker::Send;
use std::sync::Arc;

use tower_service::Service;
type Ra = Result<http::Response<http_body::Full<bytes::Bytes>>, hyper::Error>;

// 88b / request - TODO: ref more
pub struct Bridge {
    caller: Arc<Caller>,
    // scheduler: Arc<Scheduler>,
    peer: SocketAddr,
    server: SocketAddr,
    // token: CancellationToken,
}

impl Bridge {
    pub fn new(
        caller: Arc<Caller>,
        _scheduler: Arc<Scheduler>,
        peer: SocketAddr,
        server: SocketAddr,
        // token: CancellationToken,
    ) -> Self {
        Self {
            caller: caller,
            // scheduler: scheduler.clone(),
            peer,
            server,
            // token,
        }
    }
}

fn bail() -> Ra {
    Ok(Response::builder()
        .status(http::StatusCode::INTERNAL_SERVER_ERROR)
        .body(Full::new(Bytes::from("Internal Server Error")))
        .unwrap())
}

/// Servicer errors
fn bail_err(err: hyper::Error) -> Ra {
    eprintln!(
        "{} an error occurred in the servicer - {:#?}",
        "error".red(),
        err
    );
    bail()
}

/// Generic Lib Errs (internal)
fn bail_anyhow(err: anyhow::Error) -> Ra {
    eprintln!("{} an error occurred - {:#?}", "error".red(), err);
    bail()
}

/// Tokio runtime errors
fn bail_join(err: JoinError) -> Ra {
    eprintln!("{} an error occurred - {:#?}", "error".red(), err);
    bail()
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
            caller: Arc<Caller>,
            state: State,
            server: SocketAddr,
            peer: SocketAddr,
            // token: CancellationToken,
        ) -> Ra {
            // received body channel
            let (tx_bdy, rx_bdy) = crossbeam::channel::bounded::<ASGIResponse>(4);

            // outbound reponse serialization op -> see `coalesced::coalesce_from_state`
            let (tx_builder, rx_builder) = crossbeam::channel::bounded::<(Builder, Bytes)>(1);

            // spawn this on a new task so we allow the callback channel to open before py-caller
            tokio::spawn(async move {
                while let Ok(resp) = rx_bdy.recv() {
                    let mut state = state.lock().await;
                    state.0.insert(resp);
                }
                {
                    let state = state.lock().await;
                    let response =
                        coalesced::coslesce_from_state(&state, Response::builder(), true);
                    let res: Result<(), crossbeam::channel::SendError<(Builder, Bytes)>> =
                        tx_builder.send(response);
                    match res {
                        Ok(_r) => (),
                        Err(_e) => {
                            error!(
                                "{} couldnt send response to channel, cancelling due to full bail",
                                "error".red()
                            );
                            // token.cancel();
                        }
                    }
                }
            });
            // this allows functionality of `await receive()`
            let sender = Sender::new(tx_bdy);

            // get this before the body starts reading
            let asgi = asgi_from_request(&req, server, peer);

            // takes ownership of the request
            // currently a blocking op - TODO: synchronize body receivership
            let body = body::to_bytes(req.into_body()).await;
            let val = match body {
                Ok(bts) => bts,
                Err(err) => return bail_err(err),
            };
            let receiver = PyAsyncBodyReceiver::from(val);
            let join_caller = tokio::task::spawn(async move {
                let res = Python::with_gil(|py| caller.call(py, (asgi, receiver, sender)));
                // do not remove cfg stmt
                #[cfg(debug_assertions)]
                {
                    debug!("{:#?}", res);
                }
                res
            })
            .await;

            match join_caller {
                Ok(res) => {
                    match res {
                        // we dont care about the python response - nothing to receive
                        Ok(_) => {
                            match rx_builder.recv() {
                                // we have a full response
                                Ok((builder, bts)) => {
                                    return Ok(builder.body(Full::new(bts)).unwrap())
                                }
                                // an error receiving
                                Err(e) => {
                                    error!("{} occured receiving response builder - {}", "error".red(), e);
                                    return bail();
                                }
                            }
                        }
                        // internal server error, get outta here
                        Err(err) => return bail_anyhow(err),
                    }
                }
                Err(err) => bail_join(err),
            }
        }

        let hm: StateMap = StateMap::default();
        let state: State = Arc::new(Mutex::new(hm));
        Box::pin(mk_response(
            req,
            self.caller.clone(),
            state,
            self.server,
            self.peer,

            // TODO: find something to use instead
            // self.token.child_token(),
        ))
    }
}
