use async_trait::async_trait;
use colored::Colorize;
use log::info;
use pyo3::Python;
use std::time::Instant;
use tokio::sync::{
    mpsc,
    mpsc::{Receiver, Sender},
};

pub struct CallSoon {
    fu: fn(Python),
    sched_time: Instant,
}
// enum CallSoon {
//     Call { call: oneshot::Receiver<fn(&Python)>, sched_time: Instant },
// }

pub struct CallRunner {
    rx: Receiver<CallSoon>,
}

#[async_trait]
pub trait Schedule {
    async fn schedule(&self, fu: fn(Python));
}

impl CallRunner {
    pub fn new(rx: Receiver<CallSoon>) -> CallRunner {
        CallRunner { rx }
    }

    pub async fn watch(mut self) {
        #[cfg(debug_assertions)]
        info!("watching for tasks");
        while let Some(message) = self.rx.recv().await {
            #[cfg(debug_assertions)]
            info!("message {:#?}", message.sched_time);

            (message.fu)(Python::acquire_gil().python());
        }
        #[cfg(debug_assertions)]
        info!("oh no done watching");
    }
}

#[derive(Clone)]
pub struct Scheduler {
    tx: Sender<CallSoon>,
}

impl Scheduler {
    pub fn new() -> Self {
        let (tx, rx): (Sender<CallSoon>, Receiver<CallSoon>) = mpsc::channel(32);

        let actor = CallRunner::new(rx);
        tokio::spawn(actor.watch());

        Self { tx }
    }
}

#[async_trait]
impl Schedule for Scheduler {
    async fn schedule(&self, fu: fn(Python)) {
        let ok = self
            .tx
            .clone()
            .send(CallSoon {
                fu,
                sched_time: Instant::now(),
            })
            .await;
        match ok {
            Ok(_) => {}
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!(
                    "{} scheduling task due to: {:#?}",
                    "error".bold().red(),
                    e.to_string()
                );
            }
        };
    }
}
