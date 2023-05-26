use anyhow::Result;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::vec;

use std::fs;
use std::path::PathBuf;

fn get_default_asgi() -> ASGIVersions {
    ASGIVersions {
        spec_version: String::from(""),
        version: String::from("2.0"),
    }
}

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

fn pop_scope() -> ASGIScope {
    let client: String = String::from("client");
    let server: String = String::from("server");
    let scope: ASGIScope = ASGIScope {
        _type: String::from("http"),
        asgi: get_default_asgi(),
        http_version: String::from("1.1"),
        method: String::from("GET"),
        scheme: String::from("1.1"),
        path: String::from("/"),
        raw_path: vec![1, 2, 3, 4],
        query_string: vec![1, 2, 3],
        root_path: String::from("/"),
        headers: vec![],
        client: (client, 54),
        server: (server, 54),
        extensions: vec![(
            String::from("ext1"),
            vec![(String::from("ext2"), String::from("val3"))],
        )],
        subprotocols: vec![String::from("subp1")],
    };
    scope
}



// fn spvn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
//     SimpleLogger::new().env().init().unwrap();

//     Python::with_gil(|py| {
//         assert!(py.version_info() >= (3, 10));
//     });

//     m.add_function(wrap_pyfunction!(pop_scope, m)?)?;
//     Ok(())
// }
