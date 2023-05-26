use pyo3::prelude::*;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};
use simple_logger::SimpleLogger;
use std::vec;
mod cli;
mod lifespan;
use clap::Parser;

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

#[pyclass]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[pyo3(text_signature = "ASGIScope(type)")]
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

#[pyfunction]

fn pop_scope() -> PyResult<PyObject> {
    let client: String = String::from("client");
    let server: String = String::from("server");
    Python::with_gil(|py| {
        let scope = ASGIScope {
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
        Ok(pythonize(py, &scope).unwrap())
    })
}

#[pyfunction]
fn initialize(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    })
}

async fn spawn(cli: cli::cmd::Cli) {
    let spvn = lifespan::impls::new();
    log::info!("spawning");
    log::info!("{:#?}", cli);
    log::info!("{:#?}", spvn);
    // time::sleep(time::Duration::from_secs(2)).await;
    log::info!("done")
}

#[pyfunction]
fn serve(_py: Python) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let parsed: cli::cmd::Cli = cli::cmd::Cli::parse();
    let fut = spawn(parsed);
    rt.spawn(fut);
    log::info!("started.");
}

#[pymodule]
fn spvn(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    SimpleLogger::new().env().init().unwrap();

    Python::with_gil(|py| {
        assert!(py.version_info() >= (3, 10));
    });

    m.add_function(wrap_pyfunction!(pop_scope, m)?)?;
    m.add_function(wrap_pyfunction!(serve, m)?)?;
    m.add_function(wrap_pyfunction!(initialize, m)?)?;
    Ok(())
}
