use clap::Subcommand;

mod collector;
mod config;
mod download;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Config downloader.")]
    Config,
    #[command(about = "Analyze a model URL and download the model.")]
    Download,
    #[command(about = "Renew locally saved model meta information.")]
    Renew,
    #[command(about = "List all models in current directory.")]
    List,
}
