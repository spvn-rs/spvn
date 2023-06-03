use crate::implementation::set_dict_item_feedback;
use crate::implementation::{ASGIVersions, ASGI_IMPLEMENTATION};
use bytes::Bytes;
use http::{uri::Scheme, Uri, Version};
use hyper::{body::Body as IncomingBody, Request};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::Python;
use pyo3::ToPyObject;

// #[pyclass]
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

// as ugly as it is this is faster to serialize than a pyo3 custom class
impl ASGIScope {
    pub fn to_object(self, py: Python<'_>) -> pyo3::PyObject {
        let dict = PyDict::new(py);
        set_dict_item_feedback(py, &dict, "type", self._type);
        set_dict_item_feedback(py, &dict, "asgi", self.asgi.to_object(py));
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
        dict.to_object(py)
    }
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
        asgi: ASGI_IMPLEMENTATION(),
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
//
