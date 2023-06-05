use hyper::{body::Body as IncomingBody, Request, Response};
use std::pin::Pin;
use std::task::{Context, Poll};
use tower_service::Service;

pub struct LogService<S> {
    pub target: &'static str,
    pub service: S,
}

impl<S> Service<Request<IncomingBody>> for LogService<S>
where
    S: Service<Request<IncomingBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, request: Request<IncomingBody>) -> Self::Future {
        // Insert log statement here or other functionality
        println!("request = {:?}, target = {:?}", request, self.target);
        self.service.call(request)
    }
}
