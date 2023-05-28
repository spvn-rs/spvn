pub mod args;
pub(crate) mod commands;

use crate::args::{Cmds, Command, ExitStatus};
use crate::commands::serve::serve;
use anyhow::Result;
use colored::Colorize;

pub fn run(Cmds { command }: Cmds) -> Result<ExitStatus> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        {
            eprintln!(
                r#"\n{}: spvn crashed. you can report this error using https://github.com/joshua-auchincloss/spvn/issues.
                copy and paste this traceback: \n{:#?}
                "#,
                "error".red().bold(),
                info,
            );
        }
        hook(info)
    }));
    let out = match command {
        Command::Serve(args) => serve(&args),
    };
    return match out {
        Ok(status) => Ok(status),
        Err(trace) => {
            panic!("{:#?}", trace)
        }
    };
}
