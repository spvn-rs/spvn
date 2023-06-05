use crate::service::lifespan::{LifeSpan, LifeSpanError, LifeSpanState};


use pyo3::exceptions::*;
use pyo3::ffi::Py_None;
use pyo3::intern;
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use spvn_serde::{
    asgi_scope::ASGIEvent,
    event_receiver::PyASyncEventReceiver, event_sender::EventSender, ASGIType,
};
use std::{
    cmp::max,
    future::Future,
    mem::{align_of, size_of},
    ops::{Deref, DerefMut},
    ptr,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};
use tracing::debug;
use tracing::{info};

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
    state: Arc<Mutex<LifeSpanState>>,
    pub app: Box<PyObject>,
}

impl From<Py<PyAny>> for Caller {
    fn from(app: Py<PyAny>) -> Self {
        Caller {
            app: Box::new(app),
            state: Arc::new(Mutex::new(LifeSpanState::new())),
        }
    }
}

// pub type SerialToPyKwargs = fn<'a>(Python, &PyDict) -> &'a PyDict;

// static pt: SerialToPyKwargs = || {

// }
// pub fn passthru<'a>(py: Python<'a>, dict: &'a PyDict) -> &'a PyDict {
//     dict
// }

impl Caller {
    fn run_until_complete(&self) {}
    /// Expected events:
    /// 1. Send `{ type: lifespan ... }` to app
    /// 2. App requests 1st receive, which is for passing some blocking awaitable on startup
    /// 3. The app does its start ops, and sends `{type: lifespan.startup.complete ...}`
    /// 4. The app takes context within a new loop, which spawns a separate context until the next
    /// receive event occurs (`await receive()`)
    /// 5. The app then reports `{type: lifespan.startup.complete ...}`
    fn create_lifespan_handler(
        &self,
        _evt: ASGIType,
        _descr: &str,
        _on_fail: LifeSpanError,
    ) -> Result<(), LifeSpanError> {
        let _scoped = std::thread::scope(|s| {
            let (tx, rx) = crossbeam::channel::bounded::<ASGIEvent>(2);
            let (tx_cb, rx_cb) = crossbeam::channel::bounded::<Option<ASGIEvent>>(2);
            s.spawn(move || {
                while let Ok(rec) = rx.recv() {
                    let r = tx_cb.send(Some(rec));
                    info!("{:#?}", r);
                }
                let send = tx_cb.send(None);
                info!("{:#?}", send);

                info!("send complete");
            });
            s.spawn(|| {
                let send = EventSender::new(tx);
                let scope = ASGIEvent::from(ASGIType::Lifespan);
                let receive = PyASyncEventReceiver::new(ASGIEvent::from(ASGIType::Lifespan));

                let _res = Python::with_gil(|py| {
                    let result =
                    self.app
                        .call(py, (scope.to_object(py), receive, send.into_py(py)), None);
                let awa = match result {
                    Ok(succ) => succ,
                    Err(e) => panic!("{:#?}", e),
                };


                let hasawait = match  awa.call_method0(py, intern!(py, "__await__")) {
                    Ok(awaitable) => awaitable,
                    Err(_e)=>{
                        debug!("the provided lifespan is not asgi3 compliant, remediate this by adding `__await__` to your app");
                        return 
                    }
                };
                info!("await obj {:#?}", hasawait);
                let mut iterable = match hasawait.call_method0(py, "__next__"){
                    Ok(iterable) => iterable ,
                    Err(_e) => {
                        debug!("the provided lifespan is not asgi3 compliant, the returned coroutine_wrapper is missing __next__ attribute");
                        return 
                    }
                };

                loop {
                    let _ = std::thread::sleep(Duration::from_millis(1000));
                    let it = iterable.call_method0(py, intern!(py, "__next__"));
                    match it {
                        Ok(iter) => {
                            iterable = iter;
                        }
                        Err(e) => {
                            info!("err received in lifespan {:#?}", e);
                            break;
                        }
                    }
                }
                });
            });

            let rec: Option<ASGIEvent> = rx_cb.recv().unwrap();
            info!("{:#?}", rec);
            info!("rec complete");
        });
        Ok(())
    }
}

impl LifeSpan for Caller {
    fn wait_anon(&mut self, on_err: LifeSpanError) -> Result<(), LifeSpanError> {
        let rec = self.create_lifespan_handler(ASGIType::Lifespan, "lifecycle", on_err);
        match rec {
            Ok(_rec) => Ok(()),
            Err(s) => Err(s),
        }
    }
    fn wait_shutdown(&mut self) -> Result<(), LifeSpanError> {
        self.state.lock().unwrap().wait_shutdown();
        Ok(())
    }
    fn wait_startup(&self) -> Result<(), LifeSpanError> {
        let rec = self.create_lifespan_handler(
            ASGIType::LifecycleStartup,
            "start",
            LifeSpanError::LifeSpanStartFailure,
        );
        match rec {
            Ok(_r) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

pub trait Call<T>
where
    T: IntoPy<Py<PyTuple>>,
{
    fn call(&self, py: Python, base: T) -> anyhow::Result<()>;
}

fn process_async(
    py: Python,
    awaitable: PyObject,
) -> Result<(Option<PyObject>, Option<&PyException>), PyErr> {
    info!("async call");
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

    let py_result: Option<Result<PyObject, PyErr>>;
    loop {
        match await_result.call0(py) {
            Ok(o) => {
                info!("actual yield, might need to break early");
                py_result = Some(Ok(o));
                break;
            }
            Err(p) => {
                py_result = Some(Ok(p.value(py).to_object(py)));
                break;
            }
        };
    }

    #[cfg(debug_assertions)]
    info!("py_result {:#?}", py_result,);

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
                        Ok(err) => {
                            eprintln!("an exception was uncaught at runtime{:#?}", err);
                            return Err(err.into());
                        }
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

impl<T> Call<T> for Caller
where
    T: IntoPy<Py<PyTuple>>,
{
    fn call(&self, py: Python, base: T) -> anyhow::Result<()> {
        // let kwargs = serialize(py, base);
        // let app =
        let result = self.app.call(py, base, None);
        let awa = match result {
            Ok(succ) => succ,
            Err(e) => panic!("{:#?}", e),
        };
        let hasawait = awa.getattr(py, "__await__");

        let hasawait = match hasawait {
            Ok(toawait) => {
                let asyncres = process_async(py, toawait);
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
        info!("post await {:#?}", hasawait);
        let _obj = match hasawait {
            Ok(obj) => info!("{:#?}", obj),
            Err(obj) => return Err(obj.into()),
        };
        anyhow::Ok(())
    }
}

struct CallState<T>
where
    T: IntoPy<Py<PyTuple>>,
{
    caller: Arc<SyncSafeCaller>,
    args: Mutex<T>,
    calling: bool,
    result: Option<anyhow::Result<()>>,
    waker: Option<core::task::Waker>,
}

struct CallerFuture<T>
where
    T: IntoPy<Py<PyTuple>>,
{
    state: Arc<Mutex<CallState<T>>>,
}

// impl<T> Future for CallerFuture<T>
// where
//     T: IntoPy<Py<PyTuple>>,
// {
//     type Output = anyhow::Result<()>;
//     fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
//         let state = self.state.lock();
//         if self.state.is_poisoned() {
//             return Poll::Ready(Err(anyhow::Error::msg("call state poisoned")));
//         }
//         let rt = &state.unwrap();
//         if state.unwrap().calling {
//             return Poll::Pending;
//         }
//         Poll::Ready(state.unwrap().result.unwrap())
//     }
// }

impl<T> CallerFuture<T>
where
    T: IntoPy<Py<PyTuple>>,
{
    /// Create a new `TimerFuture` which will complete after the provided
    /// timeout.
    pub fn new(caller: Arc<SyncSafeCaller>, base: T) -> Self {
        let shared = Arc::new(Mutex::new(CallState {
            caller: caller.clone(),
            calling: true,
            result: None,
            waker: None,
            args: Mutex::new(base),
        }));
        let state = shared.clone();
        let _join_caller = std::thread::spawn(|| {
            // let mut st = state.lock().unwrap();
            // let args: std::sync::MutexGuard<T> = st.args.lock().unwrap();

            // let res = Python::with_gil(|py| st.caller.call(py, args.into()));
            // st.calling = false;
            // if let Some(waker) = st.waker.take() {
            //     waker.wake()
            // };
        });
        // std::thread::spawn(|| {
        //     let mut st = state.lock().unwrap();
        //     let res = Python::with_gil(|py| py.allow_threads(|| st.caller.call(py, base.as_ref())));
        //     st.calling = false;
        //     if let Some(waker) = st.waker.take() {
        //         waker.wake()
        //     };
        // });
        Self { state }
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
