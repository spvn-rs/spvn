use crate::args::ExitStatus;
use ::spvn::spvn::SpvnCfg;
use ::spvn::spvn::{BindArguments, BindMethods, HttpScheme, SecScheme, Spvn};
use anyhow::Result;
use clap::{ArgAction, Args};
use colored::Colorize;
use core::clone::Clone;
use cpython::Python;
use cpython::{py_fn, PyDict, PyNone, PyResult};
use log::info;
use notify::event;
use notify::Watcher;
use spvn::handlers::tasks::Schedule;
use spvn_caller::PySpawn;
use spvn_caller::Spawn;
use spvn_cfg::ASGIScope;
use spvn_serde::ToPy;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::time::sleep;

use std::sync::Arc;

use std::{env, path::PathBuf};

use tokio_rustls::rustls::ServerConfig as TlsConfig;

#[derive(Debug, Args)]
pub struct ServeArgs {
    // The TCP port to bind to
    #[arg(conflicts_with = "socket_bind", conflicts_with = "unix_bind")]
    pub tcp_bind: Option<String>,

    // The UNIX port to bind to
    #[arg(conflicts_with = "socket_bind", conflicts_with = "tcp_bind")]
    pub unix_bind: Option<String>,

    // The SOCKET to bind to
    #[arg(conflicts_with = "unix_bind", conflicts_with = "tcp_bind")]
    pub socket_bind: Option<String>,

    // The target "module.file:attr" to inject wrappings into
    #[arg(short, long, value_name = "FILE")]
    pub target: String,

    #[arg(long)]
    pub n_threads: Option<usize>,

    // Bind a static port and reload on changes
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub watch: Option<bool>,

    // verbose procedures
    #[arg(short, long, env = "SPVN_VERBOSE_PROC")]
    pub verbose: bool,

    // path to ssl server certificates
    #[arg(long, env = "SPVN_SSL_CERT_FILE")]
    pub ssl_cert_file: Option<PathBuf>,

    // path to ssl server keys
    #[arg(long, env = "SPVN_SSL_KEY_FILE")]
    pub ssl_key_file: Option<PathBuf>,

    #[cfg(not(windows))]
    // unix user
    #[arg(long)]
    pub user: Option<String>,

    #[cfg(not(windows))]
    // proc dir (must have +x perm on UNIX)
    #[arg(long, env = "PROC_DIR")]
    pub proc_dir: Option<PathBuf>,
}

impl From<&ServeArgs> for BindArguments {
    fn from(value: &ServeArgs) -> Self {
        if value.tcp_bind != None {
            let bind: String = value.tcp_bind.clone().unwrap();
            Self {
                bind,
                mtd: BindMethods::BindTcp,
            }
        } else if value.unix_bind != None {
            let bind: String = value.unix_bind.clone().unwrap();
            Self {
                bind,
                mtd: BindMethods::BindUnix,
            }
        } else if value.socket_bind != None {
            let bind: String = value.socket_bind.clone().unwrap();
            Self {
                bind,
                mtd: BindMethods::BindSocket,
            }
        } else {
            let bind: String = String::from("localhost:8000");
            Self {
                bind,
                mtd: BindMethods::BindTcp,
            }
        }
    }
}

impl From<&ServeArgs> for SecScheme {
    fn from(_value: &ServeArgs) -> Self {
        Self::NoTLS
    }
}

impl From<&ServeArgs> for HttpScheme {
    fn from(_value: &ServeArgs) -> Self {
        HttpScheme::Http11
    }
}

#[derive(Debug, Clone)]
pub struct Arguments {
    pub bind: BindArguments,
    pub sec_scheme: SecScheme,
    pub http_scheme: HttpScheme,
    pub target: String,
    watch: bool,
    ssl_cert_path: Option<PathBuf>,
    ssl_key_file: Option<PathBuf>,
    n_threads: usize,
}

#[derive(Debug, Clone)]
pub struct Overrides {}

impl ServeArgs {
    pub fn tree(&self) -> (Arguments, Overrides) {
        (
            Arguments {
                bind: BindArguments::from(self),
                http_scheme: HttpScheme::from(self),
                sec_scheme: SecScheme::from(self),
                target: self.target.clone(),
                watch: self.watch.unwrap_or(false),
                ssl_cert_path: self.ssl_cert_file.to_owned(),
                ssl_key_file: self.ssl_key_file.to_owned(),
                n_threads: self.n_threads.unwrap_or(1),
            },
            Overrides {},
        )
    }
}

trait To<T> {
    fn to(&self) -> T;
}

trait Merge<T> {
    fn merge(&mut self, other: T);
}

impl Merge<Overrides> for Arguments {
    fn merge(&mut self, _other: Overrides) {}
}

impl Into<SpvnCfg> for Arguments {
    fn into(self) -> SpvnCfg {
        let mut tls: Option<Arc<TlsConfig>> = None;
        let when = || {
            Some(spvn_cfg::tls_config(
                self.ssl_key_file.as_ref().expect("no ssl keyfile given"),
                self.ssl_cert_path.as_ref().expect("no ssl certfile given"),
            ))
        };
        match self.sec_scheme {
            SecScheme::NoTLS => {}
            SecScheme::TLSv12 => tls = when(),
            SecScheme::TLSv13 => tls = when(),
        }

        SpvnCfg { tls, n_threads: self.n_threads }
    }
}

pub fn serve(config: &ServeArgs) -> Result<ExitStatus> {
    let (arguments, overrides) = config.tree();
    let arguments = arguments.to_owned();
    let overrides = overrides.to_owned();

    #[cfg(debug_assertions)]
    {
        println!("{:#?} {:#?}", arguments, overrides);
    }

    let tgt: &str = arguments.target.as_str();
    env::set_var("SPVN_SRV_TARGET", tgt);
   
   
   
    if arguments.watch {
        let mut watcher = notify::recommended_watcher(
            |res: std::result::Result<notify::Event, notify::Error>| match res {
                Ok(event) => {
                    match event.kind {
                        notify::EventKind::Modify(event::ModifyKind::Metadata(_)) => {
                            println!("{} meta created... reloading", "info".blue())
                        }
                        notify::EventKind::Create(event::CreateKind::File) => {
                            println!("{} file created... reloading", "info".blue())
                        }
                        notify::EventKind::Modify(event::ModifyKind::Data(_)) => {
                            println!("{} file changed... reloading", "info".blue())
                        }

                        // notify::EventKind::Other(_) => { /* ignore meta events */ }
                        _ => { /* something else changed */ }
                    }
                    println!("event: {:?}", event)
                }
                Err(e) => println!("watch error: {:?}", e),
            },
        )?;
        let bi = watcher.watch(
            std::path::Path::new("CHANGELOG.md"),
            notify::RecursiveMode::NonRecursive,
        );

        #[cfg(debug_assertions)]
        info!("{:#?}", bi)
    }

    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    // rt.shutdown_background()
    rt.block_on(async {
        let cfg: SpvnCfg = arguments.into();
        let mut own: Spvn = cfg.into();
        own.service().await;
        // own.schedule(|py| {
        //     py.eval("print('called')", None, None);
        // });
    });

    // let mut caller = PySpawn::new();
    // caller.spawn(arguments.n_threads);
    // info!("{}", tgt);
    // let st = std::time::Instant::now();

    // caller.call(|py| {
    //     let scope = ASGIScope::mock();
    //     let kwargs = PyDict::new(py);
    //     fn send(py: Python, scope: PyDict) -> PyResult<PyNone> {
    //         #[cfg(debug_assertions)]
    //         info!("{:#?}", scope.items(py));
    //         Ok(PyNone)
    //     }

    //     fn receive(_: Python) -> PyResult<Vec<u8>> {
    //         Ok(vec![1, 2, 3])
    //     }
    //     kwargs.set_item(py, "scope", scope.to(py));
    //     kwargs.set_item(py, "send", py_fn!(py, send(scope: PyDict)));
    //     kwargs.set_item(py, "receive", py_fn!(py, receive()));
    //     kwargs
    // });
    // let end = std::time::Instant::now();

    // info!("call time: {:#?}", end.duration_since(st));
    Result::Ok(ExitStatus::Success)
}
