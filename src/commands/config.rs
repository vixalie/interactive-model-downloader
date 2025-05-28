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
    #[command(name = "enable-proxy", about = "Switch whether to use a proxy server.")]
    EnableProxy {
        #[arg(help = "Proxy enable state.")]
        flag: Option<bool>,
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

pub fn process_config_options(options: &ConfigOptions) {
    match &options.action {
        ConfigAction::Get { action } => show_config(action),
        ConfigAction::Set { action } => set_config(action),
        ConfigAction::Clear { action } => clear_config(action),
        ConfigAction::All => {}
    }
}

fn show_config(action: &ReadableContent) {
    let configuration = crate::configuration::CONFIGURATION
        .lock()
        .expect("Failed to retreive configuration.");
    match action {
        ReadableContent::CivitaiKey => {
            if let Some(key) = &configuration.civitai.api_key {
                println!("Civitai access key: {key}")
            } else {
                println!("Civitai access key has not been set.")
            }
        }
        ReadableContent::HuggingFaceKey => {
            if let Some(key) = &configuration.huggingface.api_key {
                println!("HuggingFace access key: {key}")
            } else {
                println!("HuggingFace access key has not been set.")
            }
        }
        ReadableContent::Proxy => {
            if let Some(proxy) = configuration.proxy.get_proxy_url() {
                if configuration.proxy.use_proxy {
                    println!("Using proxy server: {proxy}")
                } else {
                    println!("Downloader will not using proxy server.")
                }
            } else {
                println!("Proxy has not been set.")
            }
        }
    }
}

fn set_config(action: &WriteableContent) {
    let mut configuration = crate::configuration::CONFIGURATION
        .lock()
        .expect("Failed to access downloader configuration.");
    match action {
        WriteableContent::CivitaiKey { key } => {
            configuration
                .set_civitai_api_key(key.clone())
                .expect("Failed to save Civitai access key.");
            println!("Civitai access key has been set.")
        }
        WriteableContent::HuggingFaceKey { key } => {
            configuration
                .set_huggingface_api_key(key.clone())
                .expect("Failed to save HuggingFace access key.");
            println!("HuggingFace access key has been set.")
        }
        WriteableContent::Proxy {
            url,
            username,
            password,
        } => {
            let parsed_url = reqwest::Url::parse(&url).expect("Given proxy URL is invalid.");
            configuration
                .set_proxy(
                    parsed_url.scheme().to_string(),
                    parsed_url.host().map(|h| h.to_string()).unwrap_or_default(),
                    parsed_url.port(),
                    username.clone(),
                    password.clone(),
                )
                .expect("Failed to save proxy server configuration.");
            print!("Proxy server has been set.");
            if configuration.proxy.use_proxy {
                println!("")
            } else {
                println!(
                    " Proxy server is not enabled, you need enable it by \"enable-proxy\" command first."
                )
            }
        }
        WriteableContent::EnableProxy { flag } => {
            configuration
                .set_use_proxy(flag.unwrap_or_default())
                .expect("Failed to switch proxy server enable state.");
            println!("Download through proxy server has been activated.")
        }
    }
}

fn clear_config(action: &ReadableContent) {
    let mut configuration = crate::configuration::CONFIGURATION
        .lock()
        .expect("Failed to access downloader configuration.");
    match action {
        ReadableContent::CivitaiKey => {
            configuration
                .clear_civitai_api_key()
                .expect("Failed to clear Civitai access key.");
            println!("Civitai access key has been cleared.")
        }
        ReadableContent::HuggingFaceKey => {
            configuration
                .clear_huggingface_api_key()
                .expect("Failed to clear HuggingFace access key.");
            println!("HuggingFace access key has been cleared.")
        }
        ReadableContent::Proxy => {
            configuration
                .clear_proxy()
                .expect("Failed to clear proxy server settings.");
            println!("Proxy server settings have been cleared.")
        }
    }
}
