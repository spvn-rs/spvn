use crate::handlers::tasks::{Schedule, Scheduler};

use log::info;

use std::sync::Arc;

use async_trait::async_trait;
use tokio_rustls::rustls::ServerConfig;

pub struct SpvnCfg {
    pub tls: Option<Arc<ServerConfig>>,
}

pub struct Spvn {
    cfg: SpvnCfg,
    scheduler: Arc<Scheduler>,
}

impl Into<Spvn> for SpvnCfg {
    fn into(self) -> Spvn {
        let scheduler = Arc::new(Scheduler::new());
        Spvn {
            cfg: self,
            scheduler,
        }
    }
}

impl Spvn {
    pub async fn service(&mut self) {
        info!("starting");

        // let rt = Builder::new_multi_thread().enable_all().build().unwrap();
        // let mut reft = self.scheduler.;

        // self.schedule(|py| {
        //     py.eval("print('called')", None, None);
        // });

        // tokio::spawn(||{

        //     self.scheduler.blocking_lock().watch()
        // });

        info!("startup complete")
    }
}

#[async_trait]
impl Schedule for Spvn {
    async fn schedule(&self, fu: fn(cpython::Python)) {
        #[cfg(debug_assertions)]
        info!("scheduling");
        self.scheduler.schedule(fu).await;
    }
}
