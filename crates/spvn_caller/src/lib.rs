pub mod service;
use colored::Colorize;
use std::{env, marker::PhantomData};

use crate::service::imports::resolve_import;
use async_trait::async_trait;
use deadpool::managed;
use pyo3::prelude::*;
use service::caller::Caller;
use tracing::debug;

pub struct PySpawn {
    _data: PhantomData<Caller>,
}
pub struct PyManager {}

#[async_trait]
impl managed::Manager for PyManager {
    type Type = Caller;
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
        PySpawn {
            _data: PhantomData::default(),
        }
    }
    pub fn gen() -> Caller {
        let target = env::var("SPVN_SRV_TARGET");

        #[cfg(debug_assertions)]
        debug!("spvn target (env) {:#?}", target);

        let tgt = match target {
            Ok(st) => st,
            Err(e) => panic!("lost env var at runtime {:#?}", e),
        };
        let module = Python::with_gil(|py| resolve_import(py, tgt.as_str()));
        let caller = match module {
            Ok(asgi_app) => asgi_app,
            Err(_) => {
                panic!(
                    "{}",
                    "the caller wasnt loaded at runtime - panicking due to no backout".red()
                )
            }
        };
        caller
    }
}

#[cfg(test)]
#[allow(unused_must_use, unused_imports)]
mod tests {
    use crate::PySpawn;
    use spvn_dev::init_test_hooks;
    use spvn_serde::asgi_scope::ASGIScope;
    use std::env;
    use tracing::{debug, info};

    fn common_init_foo() {
        init_test_hooks();
        env::set_var("SPVN_SRV_TARGET", "dotest.foo:app");
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
        });
    }

    #[test]
    fn test_call_async() {
        common_init_foo();
        std::thread::spawn(move || {
            let _caller = PySpawn::new();
            let st = std::time::Instant::now();
            let end = std::time::Instant::now();
            debug!("call time: {:#?}", end.duration_since(st))
        })
        .join();
    }
}
