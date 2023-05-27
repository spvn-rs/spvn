pub mod service;
use std::env;

use crate::service::imports::resolve_import;
use anyhow::Result;
use async_trait::async_trait;
use cpython::Python;

use service::caller::{Caller, Call};
use syncpool::prelude::*;

pub static mut POOL: Option<SyncPool<PySpawn>> = None;

pub struct PySpawn {
    // py: Python,
    caller: Caller,
}

pub trait New {
    fn new(tgt: &str) -> Self;
}

impl New for PySpawn {
    fn new(tgt: &str) -> Self {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let module = resolve_import(py, tgt);
        let caller = match module {
            Ok(asgi_app) => asgi_app,
            Err(_) => panic!("panicked"),
        };
        PySpawn { caller }
    }
}

pub trait Spawn {
    fn spawn();
}

impl Call for PySpawn {
    fn call(self, py: Python) {
        self.caller.call(py)
    }
}

impl Spawn for PySpawn {
    fn spawn() {
        let gen = || {
            let target = env::var("SPVN_SRV_TARGET");
            let tgt = match target {
                Ok(st) => st,
                Err(e) => panic!("lost env var at runtime {:#?}", e),
            };

            PySpawn::new(tgt.as_str())
        };
        unsafe {
            POOL.replace(SyncPool::with_builder_and_size(4, gen));
        };
    }
}
