use async_trait::async_trait;
use cpython::_detail::ffi::Py_None;
use cpython::{py_class, py_fn, PyErr};
use cpython::{NoArgs, ObjectProtocol, PyNone};
use cpython::{PyDict, PyObject, PyResult, Python, _detail::ffi::PyAsyncMethods};
use log::info;
use spvn_serde::ToPy;
use std::{
    cmp::max,
    mem::{align_of, size_of},
    ops::{Deref, DerefMut},
    ptr,
};

use std::marker::PhantomData;

pub struct Awaitable(PyAsyncMethods);

pub struct Caller {
    pub app: PyObject,
}

impl From<PyObject> for Caller {
    fn from(app: PyObject) -> Self {
        Caller { app }
    }
}

pub type SerialToPyKwargs = fn(Python, PyDict) -> PyDict;

pub trait Call {
    fn call(&self, py: Python, serialize: SerialToPyKwargs, base: PyDict) -> anyhow::Result<()>;
    fn process_async(&self, py: Python, hasawait: PyObject) -> Result<cpython::PyObject, PyErr>;
}

impl Call for Caller {
    fn process_async(&self, py: Python, awaitable: PyObject) -> Result<cpython::PyObject, PyErr> {
        let res = awaitable.call(py, NoArgs, None);
        // coroutine = fut.__await__()

        #[cfg(debug_assertions)]
        log::info!("await result {:#?}", res);

        let awaitable = match res {
            Ok(succ) => succ,             // coroutine
            Err(e) => panic!("{:#?}", e), // called await on non awaitable
        };

        let it = awaitable.iter(py);
        let mut await_result = match it {
            Ok(succ) => succ,             // <coroutine_wrapper>
            Err(e) => panic!("{:#?}", e), // some condition we havent caught
        };

        let mut py_result: Option<Result<PyObject, cpython::PyErr>> = None;
        loop {
            py_result = match await_result.next() {
                Some(obj) => {
                    // this can 100% lead to infinite loops if there is an error traceback, so
                    // TODO: fix infinite loop conditions
                    match obj {
                        Ok(o) => Some(Ok(o)),
                        Err(p) => Some(match p.pvalue {
                            Some(v) => Ok(v),
                            None => break,
                        }),
                    }
                }
                // pass it through
                None => {
                    break;
                } // break if the coroutine has completed
            };
        }

        #[cfg(debug_assertions)]
        log::info!("py_result {:#?}", py_result,);

        let none: PyObject;
        unsafe {
            none = PyObject::from_borrowed_ptr(py, Py_None());
        }
        let res_safe: Result<PyObject, PyErr> = py_result.unwrap_or(Ok(none));
        match res_safe {
            Ok(result) => Ok(result), // if result has value, stop iteration is not called
            Err(e) => {
                info!("ERR E {:#?}", e);
                let v = e.pvalue.unwrap();
                // stop iteration
                info!("ERR E {:#?}", v);
                Ok(v) // PyNone can be returned so we unwrap
            }
        }
    }

    fn call(&self, py: Python, serialize: SerialToPyKwargs, base: PyDict) -> anyhow::Result<()> {
        let kwargs = serialize(py, base);
        let result = self.app.call(py, NoArgs, Some(&kwargs));
        let awa = match result {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };
        let hasawait = awa.getattr(py, "__await__");
        let hasawait = match hasawait {
            Ok(toawait) => self.process_async(py, toawait),
            Err(e) => Ok(awa),
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
