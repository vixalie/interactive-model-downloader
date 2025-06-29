use clap::Subcommand;

mod collector;
mod config;
mod download;
mod renew;

pub use config::process_config_options;
pub use download::process_download_options;
pub use renew::process_model_meta_renew;

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Config downloader.")]
    Config(config::ConfigOptions),
    #[command(about = "Analyze a model URL and download the model.")]
    Download(download::DownloadOptions),
    #[command(about = "Renew locally saved model meta information.")]
    Renew(renew::RenewOptions),
    #[command(about = "Scan all models in current directory, complete model meta information.")]
    Scan,
    #[command(about = "List all models in current directory.")]
    List,
}
