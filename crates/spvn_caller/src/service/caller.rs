// use cpython::_detail::ffi::Py_None;
// use cpython::{PyErr};
// use cpython::{NoArgs, ObjectProtocol};
// use cpython::{PyDict, PyObject, Python, _detail::ffi::PyAsyncMethods};

use log::info;
use pyo3::exceptions::*;
use pyo3::ffi::Py_None;
use pyo3::prelude::*;

use pyo3::types::PyTuple;

use std::{
    cmp::max,
    future::Future,
    mem::{align_of, size_of},
    ops::{Deref, DerefMut},
    ptr,
    task::Poll,
};

use std::marker::PhantomData;

pub struct CallFuture<'a, T> {
    iterating: bool,
    data: Option<&'a T>,
}

impl<'a, T> Future for CallFuture<'a, T> {
    type Output = &'a T;
    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        Poll::Ready(self.data.unwrap())
    }
}

pub struct Caller {
    pub app: Box<PyObject>,
}

impl From<Py<PyAny>> for Caller {
    fn from(app: Py<PyAny>) -> Self {
        Caller { app: Box::new(app) }
    }
}

// pub type SerialToPyKwargs = fn<'a>(Python, &PyDict) -> &'a PyDict;

// static pt: SerialToPyKwargs = || {

// }
// pub fn passthru<'a>(py: Python<'a>, dict: &'a PyDict) -> &'a PyDict {
//     dict
// }

pub trait Call {
    fn call(&self, py: Python, base: impl IntoPy<Py<PyTuple>>) -> anyhow::Result<()>;
    fn process_async(
        &self,
        py: Python,
        hasawait: PyObject,
    ) -> Result<(Option<PyObject>, Option<&PyException>), PyErr>;
}

impl Call for Caller {
    fn process_async(
        &self,
        py: Python,
        awaitable: PyObject,
    ) -> Result<(Option<PyObject>, Option<&PyException>), PyErr> {
        // coroutine = fut.__await__()
        let res = awaitable.call(py, (), None);

        let awaitable = match res {
            Ok(succ) => succ,             // coroutine
            Err(e) => panic!("{:#?}", e), // called await on non awaitable
        };

        let it = awaitable.getattr(py, "__next__");
        let await_result = match it {
            Ok(succ) => succ,             // <coroutine_wrapper>
            Err(e) => panic!("{:#?}", e), // some condition we havent caught
        };

        let mut py_result: Option<Result<PyObject, PyErr>> = None;
        let mut n = 0;
        loop {
            n += 1;
            py_result = match await_result.call0(py) {
                Ok(o) => Some(Ok(o)),
                Err(p) => {
                    py_result = Some(Ok(p.value(py).to_object(py)));
                    break;
                }
            };

            // #[cfg(debug_assertions)]
            // info!("loop {}", n)
        }

        #[cfg(debug_assertions)]
        log::info!("py_result {:#?}", py_result,);

        let none: PyObject;
        unsafe {
            none = PyObject::from_borrowed_ptr(py, Py_None());
        }
        let res_safe: Result<PyObject, PyErr> = py_result.unwrap_or(Ok(none));
        match res_safe {
            Ok(result) => {
                #[cfg(debug_assertions)]
                info!("result is ok {:#?}", result);

                let o = result.downcast::<PyStopIteration>(py);
                let asyncok = match o {
                    Ok(o) => o,
                    Err(e) => {
                        info!("{} {}", e.to_string(), result.to_string());
                        let o = result.downcast::<PyException>(py);
                        info!("cast into exception {:#?}", o);
                        match o {
                            Ok(err) => panic!("{:#?}", err),
                            Err(ohno) => panic!("{:#?}", ohno),
                        }
                    }
                };

                #[cfg(debug_assertions)]
                info!("{}", asyncok);

                // let err: Result<PyStopAsyncIteration, PyErr> = result.to_object(py).convert(py);
                // PyStopAsyncIteration::from(result.to_object(py));
                let _value = result.getattr(py, "value");
                info!("result is ok {:#?}", result);

                Ok((Some(result), None))
            } // if result has value, stop iteration is not called
            Err(e) => {
                info!("ERR E {:#?}", e);
                let v = e.value(py).to_object(py);
                // stop iteration
                info!("ERR E {:#?}", v);
                Ok((Some(v), None)) // PyNone can be returned so we unwrap
            }
        }
    }

    fn call(&self, py: Python, base: impl IntoPy<Py<PyTuple>>) -> anyhow::Result<()> {
        // let kwargs = serialize(py, base);
        // let app =
        let result = self.app.to_object(py).call(py, base, None);
        let awa = match result {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };
        let hasawait = awa.getattr(py, "__await__");

        let hasawait = match hasawait {
            Ok(toawait) => {
                let asyncres = self.process_async(py, toawait);
                match asyncres {
                    Ok((result, exception)) => {
                        if exception.is_some() {
                            eprintln!("{:#?}", exception);
                            // return Result::Err("oh no");
                        }
                        Ok(result.unwrap())
                    }
                    Err(runtime_err) => Err(runtime_err),
                }
            }
            Err(_e) => Ok(awa),
        };
        #[cfg(debug_assertions)]
        log::info!("post await {:#?}", hasawait);
        match hasawait {
            Ok(obj) => info!("{:#?}", obj),
            Err(obj) => panic!("{:#?}", obj),
        };
        anyhow::Ok(())
    }
}

#[derive(Clone, Copy)]
pub struct SyncSafeCaller {
    ptr: std::ptr::NonNull<Caller>,
    _data: PhantomData<Caller>,
}

impl SyncSafeCaller {
    pub fn new(caller: Caller) -> Self {
        let mut memptr: *mut Caller = ptr::null_mut();
        unsafe {
            let ret = libc::posix_memalign(
                (&mut memptr as *mut *mut Caller).cast(),
                max(align_of::<Caller>(), size_of::<usize>()),
                size_of::<Caller>(),
            );
            assert_eq!(ret, 0, "Failed to allocate or invalid alignment");
        };
        let ptr = { ptr::NonNull::new(memptr).expect("posix_memalign should have returned 0") };
        unsafe {
            ptr.as_ptr().write(caller);
        }
        Self {
            ptr,
            _data: PhantomData::default(),
        }
    }
}

impl Deref for SyncSafeCaller {
    type Target = Caller;
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl DerefMut for SyncSafeCaller {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

unsafe impl Send for SyncSafeCaller {}
unsafe impl Sync for SyncSafeCaller {}
