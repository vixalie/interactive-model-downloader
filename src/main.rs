use clap::Parser;

mod commands;
mod configuration;

#[derive(Parser)]
#[command(
    name = "IMD",
    author = "Vixalie",
    version = "0.1.0",
    about = "IMD is a tool for convience downloading Civitai and HuggingFace models."
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<commands::Commands>,
}

#[tokio::main]
async fn main() {
    let _cli = Cli::parse();
}
