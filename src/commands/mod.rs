use clap::Subcommand;

mod collector;
mod config;
mod download;

pub use config::process_config_options;
pub use download::process_download_options;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Config downloader.")]
    Config(config::ConfigOptions),
    #[command(about = "Analyze a model URL and download the model.")]
    Download(download::DownloadOptions),
    #[command(about = "Renew locally saved model meta information.")]
    Renew,
    #[command(about = "List all models in current directory.")]
    List,
}
