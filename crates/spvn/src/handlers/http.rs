use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use cpython::{PyDict, Python};
use futures::lock::Mutex;
use futures::Future;
use http_body::Full;
use hyper::{body::Body as IncomingBody, Request, Response};
use spvn_caller::service::caller::Call;
use spvn_caller::{service::caller::SyncSafeCaller, PySpawn};
use spvn_cfg::{asgi_from_request, ASGIScope};
use spvn_serde::ToPy;
use std::sync::Arc;
use tower_service::Service;

pub struct Bridge {
    pub caller: Arc<Mutex<SyncSafeCaller>>,
}

impl Service<Request<IncomingBody>> for Bridge {
    type Response = Response<Full<Bytes>>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<IncomingBody>) -> Self::Future {
        async fn mk_response(
            req: Request<IncomingBody>,
            caller: Arc<Mutex<SyncSafeCaller>>,
        ) -> Result<Response<Full<Bytes>>, hyper::Error> {
            // let py = ;
            let base = PyDict::new(Python::acquire_gil().python());
            let scope = asgi_from_request(&req);
            base.set_item(
                Python::acquire_gil().python(),
                "scope",
                scope.to(Python::acquire_gil().python()),
            );
            caller
                .lock()
                .await
                .call(Python::acquire_gil().python(), |py, base| base, base);

            Ok(Response::builder()
                .body(Full::new(Bytes::from("tada")))
                .unwrap())
        }
        let caller = self.caller.clone();
        Box::pin(mk_response(req, caller))
    }
}
