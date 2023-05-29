use crate::handlers::tasks::{Schedule, Scheduler};

use cpython::Python;
use hyper::server::conn::Http;
use log::info;
use spvn_caller::service::caller::SyncSafeCaller;
use spvn_caller::{PySpawn, Spawn};

use crate::startup::startup_message;
use async_trait::async_trait;
use bytes::Bytes;
use futures::executor;
use http_body::Full;
use tokio_rustls::rustls::ServerConfig;

use crate::handlers::http::Bridge;
use colored::Colorize;
use futures::future::{BoxFuture, FutureExt};
use futures::lock::Mutex;
use hyper::{Request, Response};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::{Accept, TlsAcceptor};

use tokio::io::{AsyncRead, AsyncWrite};

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
            cfg: self,
            scheduler,
        }
    }
}

impl Spvn {
    /// starts a service & blocks until signal received to shut down
    pub async fn service(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr: SocketAddr = ([127, 0, 0, 1], 8000).into();
        let listener = TcpListener::bind(addr).await?;
        // let mut caller = PySpawn::new();
        // caller.spawn(self.cfg.n_threads);

        let bi = Arc::new(Mutex::new(PySpawn::gen()));

        if !self.cfg.tls.is_none() {
            startup_message(addr, true);
            let acceptor = TlsAcceptor::from(self.cfg.tls.as_ref().unwrap().clone());
            Spvn::loop_tls(listener, acceptor, bi).await
        } else {
            startup_message(addr, false);
            Spvn::loop_passthru(listener, bi).await
        }
    }

    async fn loop_tls(
        listener: TcpListener,
        acceptor: TlsAcceptor,
        bi: Arc<Mutex<SyncSafeCaller>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let (stream, peer_addr) = listener.accept().await?;
            let acceptor = acceptor.clone();
            let bi = bi.clone();
            let fut = async move {
                let stream = acceptor.accept(stream).await?;
                if let Err(err) = Http::new()
                    .serve_connection(stream, Bridge { caller: bi.clone() })
                    .await
                {
                    println!("Failed to serve connection: {:?}", err);
                }

                Ok(()) as std::io::Result<()>
            };
            tokio::task::spawn(fut);
        }
    }

    async fn loop_passthru(
        listener: TcpListener,
        bi: Arc<Mutex<SyncSafeCaller>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        loop {
            let (stream, _) = listener.accept().await?;
            let bi = bi.clone();
            let fut = async move {
                if let Err(err) = Http::new()
                    .serve_connection(stream, Bridge { caller: bi.clone() })
                    .await
                {
                    println!("Failed to serve connection: {:?}", err);
                }

                Ok(()) as std::io::Result<()>
            };
            tokio::task::spawn(fut);
        }
    }

    /// add a callback to the task scheduler
    pub fn schedule(&mut self, fu: fn(cpython::Python)) {
        #[cfg(debug_assertions)]
        info!("scheduling");
        executor::block_on(self.scheduler.schedule(fu));
    }
}
