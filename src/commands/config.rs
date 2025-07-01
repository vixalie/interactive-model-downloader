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
    #[command(name = "retry", about = "Retry policy configuration.")]
    Retry {
        #[arg(long, short = 'r', help = "Max retry times.")]
        max_retry: Option<u32>,
        #[arg(long, short = 'i', help = "Retry interval in seconds.")]
        interval: Option<u64>,
        #[arg[long, short = 'm', help = "Retry interval increament multiplier."]]
        multiplier: Option<f32>,
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
    #[command(name = "retry", about = "Show retry policy.")]
    Retry,
}

pub async fn process_config_options(options: &ConfigOptions) {
    match &options.action {
        ConfigAction::Get { action } => show_config(action).await,
        ConfigAction::Set { action } => set_config(action).await,
        ConfigAction::Clear { action } => clear_config(action).await,
        ConfigAction::All => show_all_config().await,
    }
}

async fn show_config(action: &ReadableContent) {
    let configuration = crate::configuration::CONFIGURATION.read().await;
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
        ReadableContent::Retry => {
            println!(
                "When action failed, will retry in {} seconds, increase {:.02}x time when continuous failing, and keep retrying in {} times.",
                configuration.backoff.initial_interval,
                configuration.backoff.multiplier,
                configuration.backoff.max_retry,
            );
        }
    }
}

async fn set_config(action: &WriteableContent) {
    let mut configuration = crate::configuration::CONFIGURATION.write().await;
    match action {
        WriteableContent::CivitaiKey { key } => {
            configuration
                .set_civitai_api_key(key.clone())
                .await
                .expect("Failed to save Civitai access key.");
            println!("Civitai access key has been set.")
        }
        WriteableContent::HuggingFaceKey { key } => {
            configuration
                .set_huggingface_api_key(key.clone())
                .await
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
                .await
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
                .await
                .expect("Failed to switch proxy server enable state.");
            println!("Download through proxy server has been activated.")
        }
        WriteableContent::Retry {
            max_retry,
            interval,
            multiplier,
        } => {
            configuration
                .set_backoff(*interval, *multiplier, *max_retry)
                .await
                .expect("Failed to save retry policy.");
            println!("Retry policy has been set.")
        }
    }
}

async fn clear_config(action: &ReadableContent) {
    let mut configuration = crate::configuration::CONFIGURATION.write().await;
    match action {
        ReadableContent::CivitaiKey => {
            configuration
                .clear_civitai_api_key()
                .await
                .expect("Failed to clear Civitai access key.");
            println!("Civitai access key has been cleared.")
        }
        ReadableContent::HuggingFaceKey => {
            configuration
                .clear_huggingface_api_key()
                .await
                .expect("Failed to clear HuggingFace access key.");
            println!("HuggingFace access key has been cleared.")
        }
        ReadableContent::Proxy => {
            configuration
                .clear_proxy()
                .await
                .expect("Failed to clear proxy server settings.");
            println!("Proxy server settings have been cleared.")
        }
        ReadableContent::Retry => {
            configuration
                .clear_backoff()
                .await
                .expect("Failed to clear retry policy.");
            println!("Retry policy has been reseted.")
        }
    }
}

async fn show_all_config() {
    let configuration = crate::configuration::CONFIGURATION.read().await;
    println!(
        "Civitai access key: {}",
        configuration
            .civitai
            .api_key
            .clone()
            .unwrap_or("[NOT SET]".to_string())
    );
    println!(
        "Hugging Face access key: {}",
        configuration
            .huggingface
            .api_key
            .clone()
            .unwrap_or("[NOT SET]".to_string())
    );
    println!(
        "Proxy Server: {}",
        configuration
            .proxy
            .get_proxy_url()
            .map(|url| url.to_string())
            .unwrap_or("[NOT SET]".to_string())
    );
    println!("Use Proxy: {}", configuration.proxy.use_proxy);
    println!(
        "When action failed, will retry in {} seconds, increase {:.02}x time when continuous failing, and keep retrying in {} times.",
        configuration.backoff.initial_interval,
        configuration.backoff.multiplier,
        configuration.backoff.max_retry,
    );
}
