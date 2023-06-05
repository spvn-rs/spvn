pub mod service;
use colored::Colorize;
use std::env;

use crate::service::imports::resolve_import;
use async_trait::async_trait;
use deadpool::managed;
use pyo3::prelude::*;
use tracing::debug;

use service::caller::SyncSafeCaller;

pub struct PySpawn {
    // pool: Option<SyncSafeCaller>,
}
pub struct PyManager {}

#[async_trait]
impl managed::Manager for PyManager {
    type Type = SyncSafeCaller;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(PySpawn::gen())
    }

    async fn recycle(&self, _: &mut Self::Type) -> managed::RecycleResult<Self::Error> {
        Ok(())
    }
}

pub type PyPool = managed::Pool<PyManager>;

impl PyManager {
    pub fn new(_size: usize) -> PyPool {
        let mgr: PyManager = PyManager {};
        let pool: managed::Pool<PyManager> = PyPool::builder(mgr).build().expect("oh no");
        pool
    }
}

impl PySpawn {
    pub fn new() -> Self {
        PySpawn {}
    }
    pub fn gen() -> SyncSafeCaller {
        let target = env::var("SPVN_SRV_TARGET");
        #[cfg(debug_assertions)]
        debug!("TARGET {:#?}", target);

        let tgt = match target {
            Ok(st) => st,
            Err(e) => panic!("lost env var at runtime {:#?}", e),
        };
        let module = Python::with_gil(|py| resolve_import(py, tgt.as_str()));
        // Python::with_gil(f)
        // let py = gil.python();
        // let module = ;
        let caller = match module {
            Ok(asgi_app) => asgi_app,
            Err(_) => {
                panic!(
                    "{}",
                    "the caller wasnt loaded at runtime - panicking due to no backout".red()
                )
            }
        };
        SyncSafeCaller::new(caller)
    }
}

pub trait Spawn {
    fn spawn(&mut self, size: usize);
}

#[cfg(test)]
#[allow(unused_must_use, unused_imports)]
mod tests {
    use crate::{PySpawn, Spawn};
    // use cpython::PyDict;
    // use cpython::{py_fn, PyNone, PyResult, Python};
    use spvn_dev::init_test_hooks;
    use spvn_serde::asgi_scope::ASGIScope;
    use std::env;
    use tracing::{debug, info};

    fn common_init_foo() {
        init_test_hooks();
        env::set_var("SPVN_SRV_TARGET", "dotest.foo:app");

        // async test app
    }

    fn common_init_bar() {
        init_test_hooks();
        env::set_var("SPVN_SRV_TARGET", "dotest.bar:app");
    }

    #[test]
    fn test_call_sync() {
        common_init_bar();
        std::thread::spawn(move || {
            let _caller = PySpawn::new();
            // caller.spawn(1);
            // caller.call(|py| {
            //     let kwargs = PyDict::new(py);
            //     kwargs
            // });
        });
    }

    #[test]
    fn test_call_async() {
        common_init_foo();
        std::thread::spawn(move || {
            let _caller = PySpawn::new();
            // caller.spawn(1);
            let st = std::time::Instant::now();
            // caller.call(|py| {
            //     // let scope = ASGIScope::mock();
            //     let kwargs = PyDict::new(py);
            //     fn send(py: Python, scope: PyDict) -> PyResult<PyNone> {
            //         #[cfg(debug_assertions)]
            //         info!("{:#?}", scope.items(py));
            //         // let res = Awaitable(PyAsyncMethods {
            //         //     am_await: fn(*mut ffi::PyObject) -> *mut ffi::PyObject {

            //         //     },
            //         //     am_aiter: (),
            //         //     am_anext: (),
            //         // });
            //         // fn finish(py: Python) -> PyResult<bool> {
            //         //     Ok(true)
            //         // // }
            //         // let _bootstrap_ok = res
            //         //     ._unsafe_inner
            //         //     .set_item(py, "__await__", py_fn!(py, finish()));

            //         // #[cfg(debug_assertions)]
            //         // info!("bootstrapped: {:#?}", _bootstrap_ok);

            //         Ok(PyNone)
            //     }

            //     fn receive(_: Python) -> PyResult<Vec<u8>> {
            //         Ok(vec![1, 2, 3])
            //     }
            //     // kwargs.set_item(py, "scope", scope.to(py));
            //     kwargs.set_item(py, "send", py_fn!(py, send(scope: PyDict)));
            //     kwargs.set_item(py, "receive", py_fn!(py, receive()));
            //     kwargs
            // });
            let end = std::time::Instant::now();
            debug!("call time: {:#?}", end.duration_since(st))
        })
        .join();
    }
}
