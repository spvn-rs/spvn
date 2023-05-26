use async_trait::async_trait;
use std::sync::Arc;
// use tokio::sync::Barrier;
use tokio::sync::Mutex;

#[async_trait]
trait Lifespan {
    fn initialize(&mut self);
    async fn handle_lifespan(&self);
    async fn wait_startup(&self);
    async fn wait_shutdown(&self);
    async fn receive(&self);
    async fn send(&self);
}

#[derive(Debug)]
pub struct LifespanState {
    started: Arc<Mutex<bool>>,
    closed: Arc<Mutex<bool>>,
}

#[async_trait]
impl Lifespan for LifespanState {
    fn initialize(&mut self) {

    }
    async fn handle_lifespan(&self) {}
    async fn wait_startup(&self) {}
    async fn wait_shutdown(&self) {}
    async fn receive(&self) {}
    async fn send(&self) {}
}

pub fn new() -> LifespanState {
    let mut state = LifespanState{
        started: Arc::new(Mutex::new(false)),
        closed: Arc::new(Mutex::new(false)),
    };
    state.initialize();
    return state
}