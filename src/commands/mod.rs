use clap::Subcommand;

mod collector;
mod config;
mod download;

#[derive(Subcommand)]
pub enum Commands {
    Config,
    Download,
    Renew,
}
