use crate::args::ExitStatus;
use ::spvn::spvn::SpvnCfg;
use ::spvn::spvn::{BindArguments, BindMethods, HttpScheme, SecScheme, Spvn};
use anyhow::Result;
use clap::{ArgAction, Args};
use colored::Colorize;
use core::clone::Clone;
use std::net::SocketAddr;

use notify::event;
use notify::Watcher;
use tracing::debug;

use tokio::runtime::Builder;

use std::sync::Arc;

use std::{env, path::PathBuf};

use tokio_rustls::rustls::ServerConfig as TlsConfig;

#[derive(Debug, Args)]
pub struct ServeArgs {
    // The TCP port to bind to
    #[arg(long)]
    pub bind: Option<SocketAddr>,

    // The target "module.file:attr" to inject wrappings into
    #[arg(value_name = "py import")]
    pub target: String,

    #[cfg(not(windows))]
    #[arg(long, conflicts_with = "cpu")]
    pub n_threads: Option<usize>,

    #[cfg(not(windows))]
    #[arg(long, conflicts_with = "n_threads",  action = ArgAction::SetTrue)]
    pub cpu: Option<bool>,

    // Bind a static port and reload on changes
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub watch: Option<bool>,

    // verbose procedures
    #[arg(short, long, env = "SPVN_VERBOSE_PROC", action = ArgAction::SetTrue)]
    pub verbose: Option<bool>,

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

    // whether to use lifespan support !!! experimental !!!
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub lifespan: Option<bool>,
}

impl From<&ServeArgs> for BindArguments {
    fn from(value: &ServeArgs) -> Self {
        if value.bind.is_some() {
            let bind = value.bind.unwrap();
            Self {
                bind,
                mtd: BindMethods::BindTcp,
            }
        } else {
            let bind: SocketAddr = ([127, 0, 0, 1], 8000).into();
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
    bind: BindArguments,
    sec_scheme: SecScheme,
    _http_scheme: HttpScheme,
    target: String,
    watch: bool,
    ssl_cert_path: Option<PathBuf>,
    ssl_key_file: Option<PathBuf>,
    lifespan: bool,
    quiet: bool,

    #[cfg(not(windows))]
    n_threads: usize,
}

#[derive(Debug, Clone)]
pub struct Overrides {}

impl ServeArgs {
    pub fn tree(&self) -> (Arguments, Overrides) {
        (
            Arguments {
                bind: BindArguments::from(self),
                _http_scheme: HttpScheme::from(self),
                sec_scheme: SecScheme::from(self),
                target: self.target.clone(),
                watch: self.watch.unwrap_or(false),
                ssl_cert_path: self.ssl_cert_file.to_owned(),
                ssl_key_file: self.ssl_key_file.to_owned(),
                lifespan: self.lifespan.unwrap_or(false),
                quiet: !self.verbose.unwrap_or(true),
                
                #[cfg(not(windows))]
                n_threads: self.n_threads.unwrap_or_else(|| {
                    if self.cpu.is_some() {
                        if self.cpu.unwrap() {
                            return num_cpus::get();
                        }
                    }
                    1
                }),
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
            Some(spvn::startup::tls::tls_config(
                self.ssl_key_file.as_ref().expect("no ssl keyfile given"),
                self.ssl_cert_path.as_ref().expect("no ssl certfile given"),
            ))
        };
        match self.sec_scheme {
            SecScheme::NoTLS => {}
            SecScheme::TLSv12 => tls = when(),
            SecScheme::TLSv13 => tls = when(),
        }

        SpvnCfg {
            tls,
            bind: self.bind,
            lifespan: self.lifespan,
            quiet: self.quiet,

            #[cfg(not(windows))]
            n_threads: self.n_threads,
        }
    }
}

pub fn serve(config: &ServeArgs) -> Result<ExitStatus> {
    let (arguments, overrides) = config.tree();
    let arguments = arguments.to_owned();
    let overrides = overrides.to_owned();

    debug!("{:#?} {:#?}", arguments, overrides);

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
            std::path::Path::new("."),
            notify::RecursiveMode::NonRecursive,
        );

        #[cfg(debug_assertions)]
        debug!("{:#?}", bi)
    }
    let rt = Builder::new_multi_thread().enable_all().build().unwrap();
    let _result = rt.block_on(async move {
        let mut handlers = Vec::new();

        #[cfg(not(windows))]
        {
            for i in 0..arguments.n_threads {
                let cfg: SpvnCfg = arguments.clone().into();
                let mut own: Spvn = cfg.into();

                let h = tokio::spawn(async move { own.service(i).await });
                handlers.push(h);
            }
        }
        #[cfg(windows)]
        {
            let cfg: SpvnCfg = arguments.clone().into();
            let mut own: Spvn = cfg.into();

            let h = tokio::spawn(async move { own.service(0).await });
            handlers.push(h);
        }
        futures::future::select_all(handlers).await
    });
    Result::Ok(ExitStatus::Success)
}
