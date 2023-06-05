use std::ops::Add;
use std::sync::atomic::{AtomicI64, AtomicUsize};
use std::sync::Arc;
use std::time::Duration;

use pyo3::prelude::*;
use pyo3::prelude::{pyclass, pymethods};
use pyo3::Python;
use tracing::info;

use crate::asgi_scope::ASGIEvent;
use colored::Colorize;
use mio::{Events, Interest, Poll, Token};
use mio_signals::{Signal, SignalSet, Signals};
use pyo3::pyclass::IterNextOutput;

#[pyclass]
pub struct PySyncEventReceiver {
    pub val: ASGIEvent,
}

#[pymethods]
impl PySyncEventReceiver {
    /// Ref back to self as an iterator
    fn __call__(slf: PyRefMut<'_, Self>, py: Python) -> Result<PyObject, PyErr> {
        Ok(slf.val.clone().to_object(py))
    }
}

const LIFETIME: Token = Token(10);

#[pyclass]
pub struct PyASyncEventReceiver {
    val: ASGIEvent,
    calls: AtomicUsize,
    signalled: bool,
    poll: Poll,
    signals: Signals,
}

impl PyASyncEventReceiver {
    pub fn new(val: ASGIEvent) -> Self {
        let mut poll = Poll::new().unwrap();
        let mut signals = Signals::new(SignalSet::all()).unwrap();
        poll.registry()
            .register(&mut signals, LIFETIME, Interest::READABLE)
            .unwrap();
        Self {
            val,
            calls: AtomicUsize::new(0),
            signalled: false,
            poll: poll,
            signals,
        }
    }
}

#[pymethods]
impl PyASyncEventReceiver {
    /// Start the polling loop, ref back to self
    fn __await__(slf: PyRefMut<'_, Self>) -> Result<PyRefMut<'_, Self>, PyErr> {
        info!("await");
        Ok(slf)
    }

    /// Ref back to self as an iterator
    fn __call__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf.calls.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        slf
    }

    /// Serialize the poller into a callable iterator for python
    /// Each iteration polls the poller to determine whether to: <br/>
    ///    * a: yield the output <br/>
    ///    * b: raise timeout error <br/>
    ///    * c: poll again (\_\_next\_\_(self))
    fn __next__(mut slf: PyRefMut<'_, Self>, py: Python) -> IterNextOutput<PyObject, PyObject> {
        if slf.calls.load(std::sync::atomic::Ordering::Acquire) == 1 {
            info!("next 1");

            // send lifecycle event to caller
            return IterNextOutput::Return(slf.val.clone().to_object(py));
        }
        let mut events = Events::with_capacity(1);
        slf.poll
            .poll(&mut events, Some(Duration::from_nanos(100)))
            .unwrap();

        if events.is_empty() {
            return IterNextOutput::Yield(slf.into_py(py));
        };
        {
            for event in events.iter() {
                match slf.signals.receive().unwrap() {
                    Some(Signal::Interrupt) => {
                        info!(
                            "{}",
                            "received SIGINT... sending lifecycle termination".red()
                        );
                    }
                    Some(Signal::Terminate) => {
                        info!(
                            "{}",
                            "received SIGINT... sending lifecycle termination".red()
                        );
                    }
                    Some(Signal::Quit) => {
                        info!(
                            "{}",
                            "received SIGINT... sending lifecycle termination".red()
                        );
                    }
                    Some(sig) => {
                        info!("{:#?} unknown signal matched", sig)
                    }
                    None => return IterNextOutput::Yield(slf.into_py(py)),
                }
            }
        }
        return IterNextOutput::Return(slf.val.clone().to_object(py));
    }
}
