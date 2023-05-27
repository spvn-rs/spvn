use crate::args::ExitStatus;
use anyhow::Result;
use clap::Args;
use core::clone::Clone;
use std::{path::PathBuf, env};
use cpython::Python;
use spvn_caller::{
    service::caller::{
        Call, Caller
    },
};
use spvn_caller::{PySpawn, Spawn, POOL};
use spvn_listen::{spawn_socket, spawn_tcp};

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

    // Bind a static port and reload on changes
    #[arg(short, long)]
    pub watch: Option<bool>,

    // verbose procedures
    #[arg(short, long, env = "SPVN_VERBOSE_PROC")]
    pub verbose: bool,

    // proc dir (must have +x perm on UNIX)
    #[arg(long, env = "PROC_DIR")]
    pub proc_dir: Option<PathBuf>,
}

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

#[derive(Debug, Clone)]

pub enum SecScheme {
    NoTLS,
    TLSv12,
    TLSv13,
}

impl From<&ServeArgs> for SecScheme {
    fn from(_value: &ServeArgs) -> Self {
        Self::NoTLS
    }
}

#[derive(Debug, Clone)]

pub enum HttpScheme {
    Http11,
    Http2,
    WebSockets,
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
            },
            Overrides {},
        )
    }
}

pub async fn spawn(config: &ServeArgs) -> Result<ExitStatus> {
    let (arguments, overrides) = config.tree();
    let arguments = arguments.to_owned();
    let overrides = overrides.to_owned();

    #[cfg(debug_assertions)]
    {
        println!("{:#?} {:#?}", arguments, overrides);
    }

    let tgt: &str = arguments.target.as_str();

    env::set_var("SPVN_SRV_TARGET", tgt);
    spvn_caller::PySpawn::spawn();

    POOL.g

    // // py.
    // caller.call(py);

    // spawn_tcp( ).await;
    Result::Ok(ExitStatus::Success)
}
