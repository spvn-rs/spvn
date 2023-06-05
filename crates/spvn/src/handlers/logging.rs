use colored::Colorize;
use hyper::{body::Body as IncomingBody, Request};
use std::task::{Context, Poll};
use tower_service::Service;
use tracing::info;

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
        info!(
            "{} - {} - {}",
            self.target.blue(),
            request.method().as_str().black(),
            request.uri().path_and_query().unwrap().as_str().blue(),
        );
        self.service.call(request)
    }
}
