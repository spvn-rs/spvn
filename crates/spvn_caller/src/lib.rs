pub mod service;
use std::{env, ops::DerefMut};

use crate::service::{imports::resolve_import, lifespan::LifeSpan};

// use cpython::{
//     ObjectProtocol, PyDict, Python,
// };

use async_trait::async_trait;
use pyo3::prelude::*;
use std::{
    cmp::max,
    mem::{align_of, size_of},
    ops::Deref,
    ptr,
};

use deadpool::managed;
use log::info;

use service::caller::SyncSafeCaller;
use std::marker::PhantomData;

pub struct PySpawn {
    pool: Option<SyncSafeCaller>,
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
        PySpawn { pool: None }
    }
    // pub async fn call(self, base: impl IntoPy<Py<PyTuple>>) -> anyhow::Result<()> {
    //     self
    //         .pool
    //         .expect("call before caller acquired")
    //         .get()
    //         .deref_mut()
    //         .call(Python::acquire_gil().python(), base)
    // }
    pub fn gen() -> SyncSafeCaller {
        let target = env::var("SPVN_SRV_TARGET");
        #[cfg(debug_assertions)]
        info!("TARGET {:#?}", target);

        let tgt = match target {
            Ok(st) => st,
            Err(e) => panic!("lost env var at runtime {:#?}", e),
        };
        let module = Python::with_gil(|py| resolve_import(py, tgt.as_str()));
        // Python::with_gil(f)
        // let py = gil.python();
        // let module = ;
        let mut caller = match module {
            Ok(asgi_app) => asgi_app,
            Err(_) => panic!("panicked"),
        };

        let startup = (caller).wait_startup();

        println!("lifespan startup complete {:#?}", startup);
        // caller.
        SyncSafeCaller::new(caller)
    }
}

pub trait Spawn {
    fn spawn(&mut self, size: usize);
}

// impl Spawn for PySpawn {
// fn spawn(&mut self, size: usize) {
//     self.pool
//         .replace(SyncPool::with_builder_and_size(size, PySpawn::gen));
// }
// }

#[derive(Clone, Copy)]
pub struct PySpawnRef {
    ptr: std::ptr::NonNull<PySpawn>,
    _data: PhantomData<PySpawn>,
}

impl PySpawnRef {
    pub fn new(spawn: PySpawn) -> Self {
        let mut memptr: *mut PySpawn = ptr::null_mut();
        unsafe {
            let ret = libc::posix_memalign(
                (&mut memptr as *mut *mut PySpawn).cast(),
                max(align_of::<PySpawn>(), size_of::<usize>()),
                size_of::<PySpawn>(),
            );
            assert_eq!(ret, 0, "Failed to allocate or invalid alignment");
        };
        let ptr = { ptr::NonNull::new(memptr).expect("posix_memalign should have returned 0") };
        unsafe {
            ptr.as_ptr().write(spawn);
        }
        Self {
            ptr,
            _data: PhantomData::default(),
        }
    }
}

impl Deref for PySpawnRef {
    type Target = PySpawn;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl DerefMut for PySpawnRef {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

unsafe impl Send for PySpawnRef {}

#[cfg(test)]
#[allow(unused_must_use, unused_imports)]
mod tests {
    use crate::{PySpawn, Spawn};
    // use cpython::PyDict;
    // use cpython::{py_fn, PyNone, PyResult, Python};
    use log::info;
    use spvn_dev::init_test_hooks;
    use spvn_serde::asgi_scope::ASGIScope;
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
            info!("call time: {:#?}", end.duration_since(st))
        })
        .join();
    }
}
