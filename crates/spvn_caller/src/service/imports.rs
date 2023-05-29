use core::result::Result::Ok;
// use cpython::{PyErr, PyString, Python};
// use cpython::{
//     PyModule, PyObject,
//     _detail::ffi::{PyObject as Py3FFIObj, PySys_GetObject},
// };
use libc::c_void;
use log::{error, info};

use pyo3::ffi::{PyObject as Py3FFIObj, PySys_GetObject, Py_None};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::types::PyDict;

use crate::service::caller;

use std::env;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;

#[allow(improper_ctypes)]
extern "C" {
    //     fn PySys_GetObject(name: *const c_char) -> *mut PyObject;
    fn PyList_Append(obj: *mut Py3FFIObj, obj2: *mut Py3FFIObj) -> c_void;
}

pub enum ImportError {
    ErrorNoAttribute,
    ErrorCouldntCanonicalize,
}

// resolves an import given mylib.foo.nested:app - where app is callable
pub fn resolve_import(py: Python, import_str: &str) -> anyhow::Result<caller::Caller, ImportError> {
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
        Err(_e) => {
            error!("the target parent path is invalid: {:}", sp[0]);
            return anyhow::Result::Err(ImportError::ErrorCouldntCanonicalize);
        }
    }
    #[cfg(debug_assertions)]
    {
        println!("resolved {:?}", resolved.to_str());
    }
    #[cfg(debug_assertions)]
    {
        env::set_var("PYTHONDEBUG", "1");
    }
    import(py, pkg, &resolved, attr)
}

// ** gets module from result - panics if the result is an err to trace the error back
fn pymod_from_result_module<'a>(py: Python, result: Result<&'a PyModule, PyErr>) -> &'a PyModule {
    #[cfg(debug_assertions)]
    info!("matching module");
    let modu = match result {
        Ok(pkg) => pkg,
        Err(err) => {
            panic!("TRACEBACK {:#?}", err.print(py));
        }
    };
    modu
}

// does the main import serialization
fn import<'a, 'b>(
    py: Python<'b>,
    pkg: &str,
    path: &PathBuf,
    attr: &str,
) -> anyhow::Result<caller::Caller, ImportError> {
    #[cfg(debug_assertions)]
    info!("source to load {:#?}", path);

    #[cfg(debug_assertions)]
    info!("package to load {:}", pkg);

    let parent = path.parent().unwrap().as_os_str().to_str().unwrap();

    let target = init_module(py, pkg, parent);

    #[cfg(debug_assertions)]
    info!("loaded target from {:#?}", target.filename());

    #[cfg(debug_assertions)]
    info!("pymodule >> {:#?}", target.name(),);

    #[cfg(debug_assertions)]
    info!("using attribute >> {:#?}", attr);

    let app = target.getattr(attr);

    #[cfg(debug_assertions)]
    info!("app loaded ok! {:#?}", app);

    match app {
        Ok(imported) => return anyhow::Result::Ok(caller::Caller::from(imported.to_object(py))),
        Err(e) => panic!("{:#?}", e),
    }
}

fn init_module<'a>(py: Python<'a>, name: &str, path: &str) -> &'a PyModule {
    let py_pt = path.to_object(py).into_ptr();
    unsafe {
        let name = CString::new("path").unwrap();
        let path = PySys_GetObject(name.as_ptr());
        #[cfg(debug_assertions)]
        info!("loaded path {:#?}", path);
        PyList_Append(path, py_pt);
        #[cfg(debug_assertions)]
        info!("append to path complete");
    }
    let result = PyModule::import(py, name);
    pymod_from_result_module(py, result)
}

#[cfg(test)]
mod tests {
    use crate::service::imports::resolve_import;
    // use crate::Python;

    // #[test]
    // fn resolve_import_works() {
    //     let gil = Python::acquire_gil();
    //     let py = gil.python();
    //     let module = resolve_import(py, "foo.bar:app");
    //     assert!(module.is_ok());
    // }
}
