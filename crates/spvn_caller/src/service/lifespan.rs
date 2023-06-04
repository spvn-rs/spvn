use std::sync::{atomic::AtomicBool, Mutex};

use pyo3::{types::PyTuple, IntoPy, Py, Python};

use super::caller::Call;
// use tokio::sync::Barrier;

#[derive(Debug)]

enum Life {
    Initialized,
    LifeStarted,
    LifeEnded,
}
#[derive(Debug)]
pub enum LifeSpanError {
    LifeSpanStartFailure,
    LifeSpanEndFailure,
}
pub trait LifeSpan // where
//     T: Call<()>,
{
    fn wait_startup(&mut self) -> Result<(), LifeSpanError>;
    fn wait_shutdown(&mut self) -> Result<(), LifeSpanError>;
}

#[derive(Debug)]
pub struct LifeSpanState {
    started: AtomicBool,
    closed: AtomicBool,
    life: Mutex<Life>,
}

impl LifeSpan for LifeSpanState {
    fn wait_startup(&mut self) -> Result<(), LifeSpanError> {
        let mut life = self.life.lock().unwrap();
        *life = Life::LifeStarted;
        Ok(())
    }
    fn wait_shutdown(&mut self) -> Result<(), LifeSpanError> {
        let mut life = self.life.lock().unwrap();
        *life = Life::LifeEnded;
        Ok(())
    }
}

impl LifeSpanState {
    pub fn new() -> Self {
        Self {
            started: AtomicBool::new(false),
            closed: AtomicBool::new(false),
            life: Mutex::new(Life::Initialized),
        }
    }
}
