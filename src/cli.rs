use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    // github uri to grab
    #[arg(short, long)]
    pub get: String,

    #[arg(short, long)]
    identity: Option<PathBuf>,
}
