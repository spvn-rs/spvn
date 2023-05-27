// use axum::{extract::FromRequest, response::Response,};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use axum::{
    body::{ Bytes},
    // extract::Request,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
};
use hyper::body::Body;

static ASGIImpl: Lazy<ASGIVersions> = Lazy::new(|| ASGIVersions {
    spec_version: String::from(SpecVersion),
    version: String::from(AsgiVersion),
});

enum ASGIType {
    // lifecycle
    LifecycleStartup,
    LifecycleShutdown,
    LifecycleStartupSuccess,
    LifecycleStartupFailure,
    LifecycleShutdownSuccess,
    LifecycleShutdownFailure,

    // http, ws
    HTTPRequest,
    WS,
}
impl ASGIType {
    fn as_str(&self) -> &'static str {
        match self {
            ASGIType::LifecycleStartup => "lifecycle.startup",
            ASGIType::LifecycleShutdown => "lifecycle.shutdown",
            ASGIType::LifecycleStartupSuccess => "lifecycle.startup.success",
            ASGIType::LifecycleStartupFailure => "lifecycle.startup.failure",
            ASGIType::LifecycleShutdownSuccess => "lifecycle.shutdown.success",
            ASGIType::LifecycleShutdownFailure => "lifecycle.shutdown.failure",
            ASGIType::HTTPRequest => "http.request",
            ASGIType::WS => "websocket",
        }
    }
    fn as_string(&self) -> String {
        String::from(self.as_str())
    }
}

static SpecVersion: &str = "2.0";
static AsgiVersion: &str = "2.0";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ASGIVersions {
    spec_version: String,
    version: String,
}


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ASGIScope {
    _type: String,
    asgi: ASGIVersions,
    http_version: String,
    method: String,
    scheme: String,
    path: String,
    raw_path: Vec<u8>,
    query_string: Vec<u8>,
    root_path: String,
    headers: Vec<(Vec<u8>, Vec<u8>)>,
    client: (String, i64),
    server: (String, i64),
    extensions: Vec<(String, Vec<(String, String)>)>,
    subprotocols: Vec<String>,
}

// #[async_trait]
// impl<S, B> FromRequest<S, B> for ASGIScope
// where
//     // these bounds are required by `async_trait`
//     B: Send + 'static,
//     S: Send + Sync,
// {
//     type Rejection = StatusCode;

//     async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
//         #[cfg(debug_assertions)]

//         let uri = req.uri();
//         let version = String::from(req.version());
//         Result<Self, Self::Rejection> {


//             (, )

//         }
//         Result::Ok(ASGIScope {
//             _type: ASGIType::HTTPRequest.as_string(),
//             asgi: *ASGIImpl,
//             method: req.method().to_string(),
//             scheme: req.uri().scheme().unwrap().as_str(),
//             http_version: String::from(version),

//         })
//     }
// }

