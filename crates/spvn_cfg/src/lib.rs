use http::{uri::Scheme, Uri, Version};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict};
// use axum::{extract::FromRequest, response::Response,};
// use axum::{
//     // body::Bytes,
//     // extract::Request,
//     // http::{Method, StatusCode},
//     // response::{IntoResponse, Response},
// };
use bytes::Bytes;
use hyper::{body::Body as IncomingBody, Request};
use log::info;
use pyo3::exceptions::*;
use rustls_pemfile::{certs, pkcs8_private_keys};
use serde::{Deserialize, Serialize};
use spvn_serde::ToPy;
use std::cell::{Ref, RefCell};
use std::fmt::Display;
use std::{collections::HashMap, fs::File, io::BufReader, path::Path, sync::Arc};
use tokio_rustls::rustls::{Certificate, PrivateKey, ServerConfig};

static SpecVersion: &str = "2.0";
static AsgiVersion: &str = "2.0";

// static ASGIImpl: Lazy<ASGIVersions> = Lazy::new(|| ASGIVersions {
//     spec_version: String::from(SpecVersion),
//     version: String::from(AsgiVersion),
// });

// "type": "http.response.body",
//             "body": self.content,
//             "headers": [
//                 (
//                     b"content-lengteh",
//                     self.content_length,
//                 )
//             ],

static ERRFLAG: &str = "___FLAG___";

#[derive(Debug)]
pub struct ASGIResponse {
    _type: ASGIType,
    body: Option<Bytes>,
    headers: Vec<(String, Vec<u8>)>,
}

#[derive(FromPyObject, Debug)]
pub struct ASGIResponsePyDict<'a> {
    #[pyo3(item("type"))]
    _type: Option<String>,
    #[pyo3(item("body"))]
    body: Option<&'a PyBytes>,
    #[pyo3(item("headers"))]
    headers: Option<Vec<(String, Vec<u8>)>>,
}

impl<'a> TryInto<ASGIResponse> for ASGIResponsePyDict<'a> {
    type Error = InvalidationRationale;

    fn try_into(self) -> Result<ASGIResponse, Self::Error> {
        if self._type.is_none() {
            #[cfg(debug_assertions)]
            {
                info!("type provided is none")
            }
            return Err(InvalidationRationale {
                message: String::from(r#"missing "type" field"#),
            });
        }
        let _type = match ASGIType::from(self._type.unwrap()) {
            Ok(typ) => typ,
            Err(_) => {
                #[cfg(debug_assertions)]
                {
                    info!("invalid asgi type provided")
                }
                return Err(InvalidationRationale {
                    message: String::from("invalid asgi type provided"),
                });
            }
        };

        let mut body: Option<Bytes> = None;

        if _type == ASGIType::HTTPResponseBody {
            if self.body.is_none() {
                #[cfg(debug_assertions)]
                {
                    info!("no body provided")
                }
                return Err(InvalidationRationale {
                    message: String::from(r#"No body provided for "http.response.body" type"#),
                });
            }
            let bts = self.body.unwrap().as_bytes();
            let v: Vec<u8> = Vec::from(bts);
            let b_sr: Bytes = Bytes::from(v);
            body = Some(b_sr);
        };

        let mut headers: Vec<(std::string::String, Vec<u8>)> = vec![];
        if !self.headers.is_none() {
            headers = self.headers.unwrap();
        }

        Ok(ASGIResponse {
            _type,
            body,
            headers,
        })
    }
}

pub struct InvalidationRationale {
    pub message: String,
}
impl std::convert::From<InvalidationRationale> for PyErr {
    fn from(err: InvalidationRationale) -> PyErr {
        PyValueError::new_err(err.message.to_string())
    }
}

fn extract_with_invalidation<'a, T: 'a>(
    dict: &Ref<&'a PyDict>,
    field: &str,
) -> Result<T, InvalidationRationale>
where
    T: FromPyObject<'a>,
{
    let tp: Option<&PyAny> = dict.get_item("type");
    if tp.is_none() {
        #[cfg(debug_assertions)]
        info!("error extracting field {:#?}", field);
        return Err(InvalidationRationale {
            message: format!("missing {} field", field),
        });
    }

    let _typ: Result<T, PyErr> = tp.unwrap().extract();
    // .extract(py);
    match _typ {
        Ok(val) => Ok(val),
        Err(r) => {
            #[cfg(debug_assertions)]
            info!("error casting to type {:#?}", r);
            Err(InvalidationRationale {
                message: format!("invalid type for field {}", field),
            })
        }
    }
}

const ASGIImpl: fn() -> ASGIVersions = || ASGIVersions {
    spec_version: String::from(SpecVersion),
    version: String::from(AsgiVersion),
};

#[derive(PartialEq, Eq, Debug)]
pub enum ASGIType {
    // lifecycle
    LifecycleStartup,
    LifecycleShutdown,
    LifecycleStartupSuccess,
    LifecycleStartupFailure,
    LifecycleShutdownSuccess,
    LifecycleShutdownFailure,

    HTTPResponseStart,
    HTTPResponseBody,

    // TODO: websockets homework
    WS,
}

impl ASGIType {
    pub fn from(value: String) -> Result<Self, ()> {
        let s = value.to_lowercase();
        let ma = match s.as_str() {
            "lifecycle.startup.success" => ASGIType::LifecycleStartupSuccess,
            "lifecycle.startup.failure" => ASGIType::LifecycleStartupFailure,

            "lifecycle.shutdown.success" => ASGIType::LifecycleShutdownSuccess,
            "lifecycle.shutdown.failure" => ASGIType::LifecycleShutdownFailure,

            "http.response.start" => ASGIType::HTTPResponseStart,
            "http.response.body" => ASGIType::HTTPResponseBody,

            // all other messages can get out of here
            _ => return Err(()),
        };

        Ok(ma)
    }
}

impl Into<&str> for ASGIType {
    fn into(self) -> &'static str {
        match self {
            ASGIType::LifecycleStartup => "lifecycle.startup",
            ASGIType::LifecycleShutdown => "lifecycle.shutdown",
            ASGIType::LifecycleStartupSuccess => "lifecycle.startup.success",
            ASGIType::LifecycleStartupFailure => "lifecycle.startup.failure",
            ASGIType::LifecycleShutdownSuccess => "lifecycle.shutdown.success",
            ASGIType::LifecycleShutdownFailure => "lifecycle.shutdown.failure",
            ASGIType::HTTPResponseStart => "http.response.start",
            ASGIType::HTTPResponseBody => "http.response.body",

            // TODO: websockets
            ASGIType::WS => "websocket",
        }
    }
}

impl Into<String> for ASGIType {
    fn into(self) -> String {
        let s: &'static str = self.into();
        String::from(s)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
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
    headers: Vec<(String, Vec<u8>)>,
    client: (String, i64),
    server: (String, i64),
    extensions: Option<Vec<(String, Vec<(String, String)>)>>,
    subprotocols: Option<Vec<String>>,
}

fn version_into_string(version: Version) -> String {
    let s = match version {
        Version::HTTP_09 => "0.9",
        Version::HTTP_10 => "1.0",
        Version::HTTP_11 => "1.1",
        Version::HTTP_2 => "2",
        Version::HTTP_3 => "3",
        _ => "unsupported", // here so the compiler doesnt whine
    };
    String::from(s)
}
fn uri_into_scheme(uri: &Uri) -> String {
    let sch = uri.scheme();
    let sh = sch.unwrap_or(&Scheme::HTTP);
    String::from(sh.as_str())
}

pub fn asgi_from_request(req: &Request<IncomingBody>) -> ASGIScope {
    // let req = req.extensions().clone().get();
    let uri = req.uri().clone();
    // let query_str = uri.query().take();
    let query = "";

    let headers: Vec<(String, Vec<u8>)> = req
        .headers()
        .iter()
        .map(|hd| -> (String, Vec<u8>) {
            let (name, value) = hd;
            (String::from(name.as_str()), value.as_bytes().to_vec())
        })
        .collect();

    let http_version = version_into_string(req.version().clone());
    let method = req.method().to_string();
    let scheme = uri_into_scheme(&uri);
    let path = String::from(uri.path());
    let raw_path = Bytes::from(uri.to_string()).to_vec();
    let query_string = Bytes::from(query).to_vec();
    let root_path = String::from(uri.path());

    ASGIScope {
        _type: String::from("http"),
        asgi: ASGIImpl(),
        http_version,
        method,
        scheme,
        path,
        raw_path,
        query_string,
        root_path,
        headers,
        client: (String::from("cli"), 1),
        server: (String::from("srv"), 1),
        extensions: Some(vec![(
            String::from("abd"),
            vec![(String::from("ext1"), String::from("ext1v"))],
        )]),
        subprotocols: Some(vec![String::from("proto1")]),
    }
}

// impl ASGIScope {
//     pub fn mock() -> Self {
//         let bt: u8 = 1;
//         let vec_bt = &[bt];
//         let extensions = Some(vec![(
//             String::from("abd"),
//             vec![(String::from("ext1"), String::from("ext1v"))],
//         )]);

//         let subprotocols = Some(vec![String::from("proto1")]);
//         ASGIScope {
//             _type: String::from("http"),
//             asgi: ASGIImpl(),
//             http_version: String::from("1.1"),
//             method: String::from("GET"),
//             scheme: String::from("GET"),
//             path: String::from("GET"),
//             raw_path: vec![1, 2, 3],
//             query_string: vec![1, 2, 3],
//             root_path: String::from(""),
//             headers: vec![("", vec_bt)],
//             client: (String::from("cli"), 1),
//             server: (String::from("srv"), 1),
//             extensions: None,
//             subprotocols: None,
//             // extensions,
//             // subprotocols,
//         }
//     }
// }

struct PObjRef(PyObject);

impl IntoPy<PyObject> for PObjRef {
    fn into_py(self, py: Python) -> PyObject {
        self.0
    }
}

trait To<T> {
    fn to(self) -> T;
}

impl<'a> ToPy<'a, &'a PyDict> for ASGIVersions {
    fn to(self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        set_dict_item_feedback(py, &dict, "spec_version", self.spec_version);
        set_dict_item_feedback(py, &dict, "version", self.version);
        dict
    }
}

fn set_dict_item_feedback<K: ToPyObject, V: ToPyObject>(py: Python, dict: &PyDict, k: K, v: V) {
    let _res = dict.set_item(k, v);
}

impl<'a> ToPy<'a, &'a PyDict> for ASGIScope {
    fn to(self, py: Python<'a>) -> &'a PyDict {
        let dict = PyDict::new(py);
        set_dict_item_feedback(py, &dict, "type", self._type);
        set_dict_item_feedback(py, &dict, "asgi", self.asgi.to(py));
        set_dict_item_feedback(py, &dict, "http_version", self.http_version);
        set_dict_item_feedback(py, &dict, "method", self.method);
        set_dict_item_feedback(py, &dict, "scheme", self.scheme);
        set_dict_item_feedback(py, &dict, "path", self.path);
        set_dict_item_feedback(py, &dict, "raw_path", self.raw_path);
        set_dict_item_feedback(py, &dict, "query_string", self.query_string);
        set_dict_item_feedback(py, &dict, "root_path", self.root_path);
        set_dict_item_feedback(py, &dict, "headers", self.headers);
        set_dict_item_feedback(py, &dict, "client", self.client);
        set_dict_item_feedback(py, &dict, "server", self.server);
        set_dict_item_feedback(py, &dict, "extensions", self.extensions);
        set_dict_item_feedback(py, &dict, "subprotocols", self.subprotocols);
        dict
    }
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

pub fn tls_config(key: impl AsRef<Path>, cert: impl AsRef<Path>) -> Arc<ServerConfig> {
    let mut key_reader = BufReader::new(File::open(key).unwrap());
    let mut cert_reader = BufReader::new(File::open(cert).unwrap());

    let key = PrivateKey(pkcs8_private_keys(&mut key_reader).unwrap().remove(0));
    let certs = certs(&mut cert_reader)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();

    let mut config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("bad certificate/key");

    config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Arc::new(config)
}
