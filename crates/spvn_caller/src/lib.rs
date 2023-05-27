pub mod service;
use std::env;

use crate::service::imports::resolve_import;
use anyhow::Result;
use cpython::Python;
use spvn_cfg::ASGIScope;

use service::caller::{Call, Caller};
use syncpool::prelude::*;

pub struct PySpawn {
    pool: Option<SyncPool<Caller>>,
}

impl PySpawn {
    pub fn new() -> Self {
        PySpawn { pool: None }
    }
    pub fn call(self, scope: ASGIScope) {
        let gil = Python::acquire_gil();
        self.pool
            .expect("call before caller acquired")
            .get()
            .call(gil.python(), scope);
    }
    pub fn gen() -> Caller {
        let target = env::var("SPVN_SRV_TARGET");
        let tgt = match target {
            Ok(st) => st,
            Err(e) => panic!("lost env var at runtime {:#?}", e),
        };
        let gil = Python::acquire_gil();
        let py = gil.python();
        let module = resolve_import(py, tgt.as_str());
        let caller = match module {
            Ok(asgi_app) => asgi_app,
            Err(_) => panic!("panicked"),
        };
        caller
    }
}

pub trait Spawn {
    fn spawn(&mut self);
}

impl Spawn for PySpawn {
    fn spawn(&mut self) {
        self.pool
            .replace(SyncPool::with_builder_and_size(4, PySpawn::gen));
    }
}
