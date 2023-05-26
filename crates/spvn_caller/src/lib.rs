use async_trait::async_trait;
use core::result::Result::Ok;
use cpython::{PyErr, PyString, Python};
use cpython::{PyModule, PyObject};
use libc::{c_char, c_void};
use log::{error, info};

use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::{env, vec};

#[allow(improper_ctypes)]
extern "C" {
    fn PySys_GetObject(name: *const c_char) -> *mut PyObject;
    fn PyList_Append(obj: *mut PyObject, obj2: cpython::PyString) -> c_void;
}

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

pub enum ImportError {
    ErrorNoAttribute,
    ErrorCouldntCanonicalize,
}

pub fn resolve_import(import_str: &str) -> anyhow::Result<Caller, ImportError> {
    let tgt = String::from(import_str);
    let spl = tgt.split(":").collect::<Vec<&str>>();
    if spl.len() != 2 {
        error!("no attribute selected or no parent module. ensure the target is specified as `foo.bar:app`");
        return anyhow::Result::Err(ImportError::ErrorNoAttribute);
    }

    let (pkg, attr) = (spl[0], spl[1]);
    let pkgstr = String::from(pkg);
    let sp: Vec<&str> = pkgstr.split(".").collect();

    #[cfg(debug_assertions)]
    {
        println!("found package {:#?}", sp);
    }

    let found = fs::canonicalize(&PathBuf::from(sp[0]));
    let resolved: PathBuf;
    match found {
        Ok(path) => resolved = path,
        Err(e) => {
            error!("the target parent path is invalid: {:}", sp[0]);
            return anyhow::Result::Err(ImportError::ErrorCouldntCanonicalize);
        }
    }
    let gil = Python::acquire_gil();
    #[cfg(debug_assertions)]
    {
        println!("resolved {:?}", resolved.to_str());
    }
    #[cfg(debug_assertions)]
    {
        env::set_var("PYTHONDEBUG", "1");
    }
    import(gil.python(), pkg, &resolved, attr)
}

fn pymod_from_result_module(py: Python, result: Result<PyModule, PyErr>) -> PyModule {
    let modu = match result {
        cpython::_detail::Result::Ok(pkg) => pkg,
        Err(err) => {
            panic!("TRACEBACK {:#?}", err.print(py));
        }
    };
    modu
}

fn inimodule(py: Python, name: &str, path: &str) -> PyModule {
    let pypath = PyString::new(py, path);
    unsafe {
        let name = CString::new("path").unwrap();
        let path = PySys_GetObject(name.as_ptr());
        #[cfg(debug_assertions)]
        info!("loaded pyobj {:#?}", path);
        PyList_Append(path, pypath);
    }
    let result = PyModule::import(py, name);
    pymod_from_result_module(py, result)
}

#[async_trait]
trait Call {
    fn sync(&self);
    async fn asgi(&self);
}
pub struct Caller {
    pub app: PyObject,
}

#[async_trait]
impl Call for Caller {
    fn sync(&self) {}
    async fn asgi(&self) {}
}

impl From<PyObject> for Caller {
    fn from(value: PyObject) -> Self {
        Caller { app: value }
    }
}

fn import(
    py: Python,
    pkg: &str,
    path: &PathBuf,
    attr: &str,
) -> anyhow::Result<Caller, ImportError> {
    #[cfg(debug_assertions)]
    info!("source to load {:#?}", path);

    #[cfg(debug_assertions)]
    info!("package to load {:}", pkg);

    let parent = path.parent().unwrap().as_os_str().to_str().unwrap();

    let target = inimodule(py, "dotest.foo", parent);

    #[cfg(debug_assertions)]
    {
        info!("loaded target from {:#?}", target.filename(py));
    }

    #[cfg(debug_assertions)]
    info!("pymodule >> {:#?}", target.name(py),);

    #[cfg(debug_assertions)]
    info!("using attribute >> {:#?}", attr);

    let app: Result<PyObject, cpython::PyErr> = target.get(py, attr);

    #[cfg(debug_assertions)]
    info!("app loaded ok! {:#?}", app);

    match app {
        cpython::_detail::Result::Ok(imported) => {
            return anyhow::Result::Ok(Caller::from(imported))
        }
        Err(e) => panic!("{:#?}", e),
    }
}
