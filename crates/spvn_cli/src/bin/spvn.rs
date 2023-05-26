use clap::Parser;

use colored::Colorize;
use simple_logger::SimpleLogger;
use spvn_cli::args::{Cmds, ExitStatus};
use spvn_cli::run;
use std::process::ExitCode;

pub fn main() -> ExitCode {
    SimpleLogger::new().env().init().unwrap();
    let cmd = Cmds::parse();
    match run(cmd) {
        Ok(code) => code.into(),
        Err(err) => {
            #[allow(clippy::print_stderr)]
            {
                eprintln!("{}{} {err:?}", "error".red().bold(), ":".bold());
            }
            ExitStatus::Error.into()
        }
    }
}
