use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Slurm")]
#[command(author = "taybart <taybart@gmail.com>", version)]
#[command(about = "Party with slurms, download github folders", long_about = None)]
pub struct Cli {
    /// Github uri to grab
    #[arg(short, long)]
    pub get: String,

    /// Identity file to use (default ~/.ssh/id_ed25519)
    #[arg(short)]
    pub identity_file: Option<PathBuf>,
}
