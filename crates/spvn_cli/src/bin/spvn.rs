use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "spvn")]
#[command(author = "Joshua A. <joshua.auchincloss@proton.me>")]
#[command(version)]
#[command(about = "ASGI Rust Bindings", long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    pub target: String,

    pub config: Option<PathBuf>,
}
