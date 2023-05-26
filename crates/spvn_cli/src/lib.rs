use crate::args::{Cmds, Command, ExitStatus};
use crate::commands::serve::{spawn, ServeArgs};
use anyhow::Result;
use colored::Colorize;
pub mod args;
pub(crate) mod commands;

pub fn serve(args: ServeArgs) -> () {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            spawn(&args).await
        });
}

pub fn run(Cmds { command }: Cmds) -> Result<ExitStatus> {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        {
            eprintln!(r#"\n{}: oops crashed.\n"#, "error".red().bold(),);
        }
        hook(info)
    }));

    let out = match command {
        Command::Serve(args) => serve(args),
    };

    Ok(ExitStatus::Success)
}
