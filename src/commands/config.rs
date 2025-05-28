use clap::{Args, Subcommand};

#[derive(Args)]
pub struct ConfigOptions {
    #[command(subcommand, help = "Inspect or modify downloader configuration.")]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    #[command(about = "Inspect spcific configuration.")]
    Get {
        #[command(subcommand)]
        action: ReadableContent,
    },
    #[command(about = "Modify spcific configuration.")]
    Set {
        #[command(subcommand)]
        action: WriteableContent,
    },
    #[command(about = "Clear spcific configuration.")]
    Clear {
        #[command(subcommand)]
        action: ReadableContent,
    },
    #[command(about = "Show all configuration.")]
    All,
}

#[derive(Subcommand)]
pub enum WriteableContent {
    #[command(name = "civitai", about = "Operate Civitai access key.")]
    CivitaiKey {
        #[arg(help = "Civitai access key.")]
        key: String,
    },
    #[command(name = "huggingface", about = "Operate HuggingFace Access key.")]
    HuggingFaceKey {
        #[arg(help = "HuggingFace access key.")]
        key: String,
    },
    #[command(name = "proxy", about = "Operate proxy.")]
    Proxy {
        #[arg(help = "Proxy server URL.")]
        url: String,
        #[arg(long, short = 'u', help = "Username for Proxy server authentication.")]
        username: Option<String>,
        #[arg(long, short = 'p', help = "Password for Proxy server authentication.")]
        password: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ReadableContent {
    #[command(name = "civitai", about = "Show Civitai access key.")]
    CivitaiKey,
    #[command(name = "huggingface", about = "Show HuggingFace Access key.")]
    HuggingFaceKey,
    #[command(name = "proxy", about = "Show proxy.")]
    Proxy,
}
