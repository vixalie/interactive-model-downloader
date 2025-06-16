use clap::Parser;

mod civitai;
mod commands;
mod configuration;
mod downloader;
mod errors;
mod hugging_face;

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
    let cli = Cli::parse();

    match cli.command {
        Some(commands::Commands::Config(options)) => {
            commands::process_config_options(&options).await
        }
        Some(commands::Commands::Download(options)) => {
            commands::process_download_options(&options).await
        }
        _ => {}
    }
}
