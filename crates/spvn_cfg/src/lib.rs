use clap::Parser;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Parser, Debug, Deserialize)]
#[command(name = "spvn")]
#[command(author)]
#[command(version)]
#[command(about = "ASGI Rust Bindings", long_about = None)]
pub struct Config {
    pub port: i32,

    #[arg(short, long, value_name = "FILE")]
    pub target: String,

    #[arg(short, long, value_name = "CONFIG")]
    pub config_file: Option<PathBuf>,
}


pub fn parse_config()-> Config {
    let cfg = Config::parse();


    cfg
}