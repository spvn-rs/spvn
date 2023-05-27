use crate::commands::serve::ServeArgs;
use clap::{command, Parser};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "spvn")]
#[command(author)]
#[command(version)]
#[command(about = "ASGI Rust Bindings", long_about = None)]
pub struct Cmds {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Serve(ServeArgs),
}

#[derive(Copy, Clone)]
pub enum ExitStatus {
    Success,
    Failure,
    Error,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => ExitCode::from(0),
            ExitStatus::Failure => ExitCode::from(1),
            ExitStatus::Error => ExitCode::from(2),
        }
    }
}
