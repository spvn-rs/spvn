use std::sync::Mutex;

#[derive(Debug)]
enum Life {
    Initialized,
    LifeStarted,
    LifeEnded,
}
#[derive(Debug, Copy, Clone)]
pub enum LifeSpanError {
    LifeSpanStartFailure,
    LifeSpanEndFailure,
}
pub trait LifeSpan {
    fn wait_startup(&mut self) -> Result<(), LifeSpanError>;
    fn wait_shutdown(&mut self) -> Result<(), LifeSpanError>;
    fn wait_anon(&mut self, which: LifeSpanError) -> Result<(), LifeSpanError>;
}

#[derive(Debug)]
pub struct LifeSpanState {
    life: Mutex<Life>,
}

impl LifeSpan for LifeSpanState {
    fn wait_anon(&mut self, _which: LifeSpanError) -> Result<(), LifeSpanError> {
        Ok(())
    }
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
            life: Mutex::new(Life::Initialized),
        }
    }
}
