use clap::Parser;
use std::path::PathBuf;
use log::Level;
use simple_logger::SimpleLogger;


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


fn main(){
    SimpleLogger::new().env().init().unwrap();

    let cli = Cli::parse();
    log::info!("{:#?}", cli);
}