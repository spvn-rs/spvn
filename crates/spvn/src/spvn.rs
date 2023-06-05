use crate::handlers::{
    logging::LogService,
    tasks::{Schedule, Scheduler},
};

use hyper::server::conn::Http;
use tracing::debug;

use spvn_caller::{PySpawn, service::caller::Caller};

use futures::executor;
use pyo3::Python;
use tokio_rustls::rustls::ServerConfig;

use crate::handlers::http::Bridge;
use spvn_caller::service::lifespan::LifeSpan;
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
    pub bind: SocketAddr,
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
    pub bind: BindArguments,

    pub quiet: bool,

    #[cfg(feature = "lifespan")]
    pub lifespan: bool,
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
    bi: Arc<Caller>,
    scheduler: Arc<Scheduler>,
    server: SocketAddr,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (stream, peer) = listener.accept().await?;
        let acceptor = acceptor.clone();
        let bi = bi.clone();
        let scheduler = scheduler.clone();

        let service = Bridge::new(bi.clone(), scheduler.clone(), peer, server);
        if !quiet {
            let svc = LogService {
                target: "bridge",
                service,
            };
            let fut = async move {
                let stream = acceptor.accept(stream).await?;
                if let Err(err) = Http::new().serve_connection(stream, svc).await {
                    println!("Failed to serve connection: {:?}", err);
                }

                Ok(()) as std::io::Result<()>
            };
            tokio::spawn(fut);
        } else {
            let fut = async move {
                let stream = acceptor.accept(stream).await?;
                if let Err(err) = Http::new().serve_connection(stream, service).await {
                    println!("Failed to serve connection: {:?}", err);
                }
                Ok(()) as std::io::Result<()>
            };
            tokio::spawn(fut);
        }
    }
}

async fn loop_passthru(
    listener: TcpListener,
    bi: Arc<Caller>,
    scheduler: Arc<Scheduler>,
    server: SocketAddr,
    quiet: bool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let (stream, peer) = listener.accept().await?;
        let bi = bi.clone();
        let scheduler = scheduler.clone();

        let service = Bridge::new(bi.clone(), scheduler.clone(), peer, server);
        if !quiet {
            let svc = LogService {
                target: "bridge",
                service,
            };
            let fut = async move {
                if let Err(err) = Http::new().serve_connection(stream, svc).await {
                    println!("Failed to serve connection: {:?}", err);
                }

                Ok(()) as std::io::Result<()>
            };
            tokio::spawn(fut);
        } else {
            let fut = async move {
                if let Err(err) = Http::new().serve_connection(stream, service).await {
                    println!("Failed to serve connection: {:?}", err);
                }
                Ok(()) as std::io::Result<()>
            };
            tokio::spawn(fut);
        }
    }
}

impl Spvn {
    /// starts a service & blocks until signal received to shut down
    pub async fn service(
        &mut self,
        pid: usize,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = self.cfg.bind.bind;
        let listener = crate::startup::listen::spawn_so_reuse(addr).await;
        let reffed = PySpawn::gen();

        let bi: Arc<Caller> = Arc::new(reffed);
        #[cfg(feature = "lifespan")]
        {
            if self.cfg.lifespan {
                let ref_ = bi.clone();
                tokio::spawn(async move {
                    ref_.wait_startup();
                });
            }
        }

        if !self.cfg.tls.is_none() {
            crate::startup::message::startup_message(pid, addr, true);
            let acceptor = TlsAcceptor::from(self.cfg.tls.as_ref().unwrap().clone());
            loop_tls(
                listener,
                acceptor,
                bi,
                self.scheduler.clone(),
                addr,
                self.cfg.quiet,
            )
            .await
        } else {
            crate::startup::message::startup_message(pid, addr, false);
            loop_passthru(listener, bi, self.scheduler.clone(), addr, self.cfg.quiet).await
        }
    }

    /// add a callback to the task scheduler
    pub fn schedule(&mut self, fu: fn(Python)) {
        debug!("scheduling");
        executor::block_on(self.scheduler.schedule(fu));
    }
}

#[cfg(test)]
pub mod tests {
    use crate::spvn::{Spvn, SpvnCfg};

    #[cfg(feature = "lifespan")]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_integration_no_ssl() {
        spvn_dev::init_test_hooks();
        std::env::set_var("SPVN_SRV_TARGET", "dotest.bit:app");
        let cfg = SpvnCfg {
            tls: None,
            n_threads: 1,
            bind: crate::spvn::BindArguments {
                bind: ([127, 0, 0, 1], 1234).into(),
                mtd: crate::spvn::BindMethods::BindTcp,
            },
            quiet: false,
            lifespan: false,
        };
        let mut spvn: Spvn = cfg.into();
        let j1 = tokio::spawn(async move { spvn.service(1).await });

        j1.abort();
    }
}
