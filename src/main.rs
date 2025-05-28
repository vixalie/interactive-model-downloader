use clap::Parser;

mod commands;
mod configuration;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<commands::Commands>,
}

#[tokio::main]
async fn main() {
    println!("Hello, world!");
}
