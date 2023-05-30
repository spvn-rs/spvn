pub mod asgi_scope;
pub mod implementation;
pub mod sender;
pub mod receiver;
pub mod state;
pub mod call_async;
use bytes::Bytes;


use log::info;
use pyo3::{
    exceptions::*,
    prelude::*,
    types::{PyBytes},
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

pub struct InvalidationRationale {
    pub message: String,
}

impl std::convert::From<InvalidationRationale> for PyErr {
    fn from(err: InvalidationRationale) -> PyErr {
        PyValueError::new_err(err.message.to_string())
    }
}

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
