use crate::handlers::tasks::{Schedule, Scheduler};

use hyper::server::conn::Http;
use log::info;

use crate::startup::startup_message;
use spvn_caller::PySpawn;

use futures::executor;
use pyo3::Python;
use tokio_rustls::rustls::ServerConfig;

use crate::handlers::http::Bridge;

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

#[derive(Debug, Clone)]
pub enum BindMethods {
    BindTcp,
    BindUnix,
    BindSocket,
}
#[derive(Debug, Clone)]
pub struct BindArguments {
    pub bind: String,
    pub mtd: BindMethods,
}

#[derive(Debug, Clone)]
pub enum SecScheme {
    NoTLS,
    TLSv12,
    TLSv13,
}

#[derive(Debug, Clone)]

pub enum HttpScheme {
    Http11,
    Http2,
    WebSockets,
}

#[derive(Clone)]
pub struct SpvnCfg {
    pub tls: Option<Arc<ServerConfig>>,
    pub n_threads: usize,
}

pub struct Spvn {
    cfg: SpvnCfg,
    scheduler: Arc<Scheduler>,
}

impl Into<Spvn> for SpvnCfg {
    /// must have SPVN_SRV_TARGET env var set
    fn into(self) -> Spvn {
        let scheduler = Arc::new(Scheduler::new());
        Spvn {
            cfg: self.clone().to_owned(),
            scheduler,
        }
    }
}

async fn loop_tls(
    listener: TcpListener,
    acceptor: TlsAcceptor,
    bi: Arc<spvn_caller::service::caller::SyncSafeCaller>,
    scheduler: Arc<Scheduler>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (stream, _peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let bi = bi.clone();
        let scheduler = scheduler.clone();

        let fut = async move {
            let stream = acceptor.accept(stream).await?;
            if let Err(err) = Http::new()
                .serve_connection(stream, Box::pin(Bridge::new(bi.clone(), scheduler.clone())))
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }

            Ok(()) as std::io::Result<()>
        };
        tokio::spawn(fut);
    }
}

async fn loop_passthru(
    listener: TcpListener,
    bi: Arc<spvn_caller::service::caller::SyncSafeCaller>,
    scheduler: Arc<Scheduler>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (stream, _addr) = listener.accept().await?;
        let bi = bi.clone();
        let scheduler = scheduler.clone();
        let fut = async move {
            eprintln!("serving");

            if let Err(err) = Http::new()
                .serve_connection(stream, Box::pin(Bridge::new(bi, scheduler.clone())))
                .await
            {
                println!("Failed to serve connection: {:?}", err);
            }

            Ok(()) as std::io::Result<()>
        };
        tokio::spawn(fut);
    }
}

impl Spvn {
    /// starts a service & blocks until signal received to shut down
    pub async fn service(
        &mut self,
        pid: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();

        let listener = spvn_listen::spawn_so_reuse(addr).await;
        let bi: Arc<spvn_caller::service::caller::SyncSafeCaller> = Arc::new(PySpawn::gen());

        if !self.cfg.tls.is_none() {
            startup_message(pid, addr, true);
            let acceptor = TlsAcceptor::from(self.cfg.tls.as_ref().unwrap().clone());
            loop_tls(listener, acceptor, bi, self.scheduler.clone()).await
        } else {
            startup_message(pid, addr, false);
            loop_passthru(listener, bi, self.scheduler.clone()).await
        }
    }

    /// add a callback to the task scheduler
    pub fn schedule(&mut self, fu: fn(Python)) {
        #[cfg(debug_assertions)]
        info!("scheduling");
        executor::block_on(self.scheduler.schedule(fu));
    }
}
