pub mod service;
use std::{env, ops::DerefMut};

use crate::service::imports::resolve_import;

// use cpython::{
//     ObjectProtocol, PyDict, Python,
// };

use pyo3::prelude::*;

use log::info;
use pyo3::types::PyTuple;
use service::caller::{Call, SyncSafeCaller};
use syncpool::prelude::*;
pub struct PySpawn {
    pool: Option<SyncPool<SyncSafeCaller>>,
}

// fn make_sync<'life0, 'async_trait, T>(
//     &'life0 self,
//     fu: fn(Python),
// ) -> ::core::pin::Pin<
//     Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
// >
// where
//     'life0: 'async_trait,
//     T: 'async_trait,
// {
//     let call = async {};
//     // call
// }

// #[async_trait]
// trait CallAsyncRCSafe {
//     async fn async_call(
//         &self,
//         serialize: SerialToPyKwargs,
//         schedule: fn(
//             fn(Python),
//         )
//             -> Pin<Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send>>,
//     ) {
//     }
// }
// py_class!(class AsyncCallKwargs |py| {

//     data schedule: fn(
//         fn(Python),
//     ) -> Pin<
//         Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send>,
//     > ;

//     // def __new__(_cls) -> PyResult<AsyncCallKwargs> {
//     //     AsyncCallKwargs::create_instance(py,

//     //         )
//     // }

//     def call_soon(&self, fu) -> PyResult<PyNone> {
//         let schedule = |req| Box::pin(self.schedule(req));
//         let vc = fu.to_py_object(py);
//         let fut = |py| {
//             vc.call(py, NoArgs, None);
//         };
//         tokio::spawn(schedule(py));

//         Ok(PyNone)
//     }
// });

// #[async_trait]
// impl CallAsyncRCSafe for PySpawn {
//     async fn async_call(
//         &self,
//         serialize: SerialToPyKwargs,
//         schedule: fn(
//             fn(Python),
//         )
//             -> Pin<Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send>>,
//     ) {
//         let schedule = |req| Box::pin(schedule(req));

//         tokio::spawn(schedule(None));

//         self.call(|py| -> PyDict {
//             let r = PyDict::new(py);

//             r
//         });
//     }
// }

impl PySpawn {
    pub fn new() -> Self {
        PySpawn { pool: None }
    }

    pub fn call(self, base: impl IntoPy<Py<PyTuple>>) {
        let gil = Python::acquire_gil();
        let ini_result = self
            .pool
            .expect("call before caller acquired")
            .get()
            .deref_mut()
            .call(gil.python(), base);
        // .call(, serialize);

        #[cfg(debug_assertions)]
        info!("{:#?}", ini_result)
    }
    pub fn gen() -> SyncSafeCaller {
        let target = env::var("SPVN_SRV_TARGET");
        #[cfg(debug_assertions)]
        info!("TARGET {:#?}", target);

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
        SyncSafeCaller::new(caller)
    }
}

pub trait Spawn {
    fn spawn(&mut self, size: usize);
}

impl Spawn for PySpawn {
    fn spawn(&mut self, size: usize) {
        self.pool
            .replace(SyncPool::with_builder_and_size(size, PySpawn::gen));
    }
}

#[cfg(test)]
#[allow(unused_must_use, unused_imports)]
mod tests {
    use crate::{PySpawn, Spawn};
    // use cpython::PyDict;
    // use cpython::{py_fn, PyNone, PyResult, Python};
    use log::info;
    use spvn_cfg::ASGIScope;
    use spvn_dev::init_test_hooks;
    use spvn_serde::ToPy;
    use std::env;

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
            let mut caller = PySpawn::new();
            caller.spawn(1);
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
            let mut caller = PySpawn::new();
            caller.spawn(1);
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
            info!("call time: {:#?}", end.duration_since(st))
        })
        .join();
    }
}
