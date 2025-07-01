use std::time::Duration;

use backoff::{ExponentialBackoff, ExponentialBackoffBuilder};
use reqwest::{Client, ClientBuilder, Url};

use crate::configuration;

pub enum Platform {
    Civitai,
    HuggingFace,
}

pub fn detect_platform(url: &Url) -> Option<Platform> {
    match url.host_str() {
        Some(host) if host.eq_ignore_ascii_case("civitai.com") => Some(Platform::Civitai),
        Some(host) if host.eq_ignore_ascii_case("huggingface.co") => Some(Platform::HuggingFace),
        _ => None,
    }
}

pub async fn make_client() -> anyhow::Result<Client> {
    let config = crate::configuration::CONFIGURATION.read().await;
    let proxy = config.proxy.get_proxy();

    let client_builder = ClientBuilder::new().user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36").use_rustls_tls();
    let client_builder = if let Some(proxy) = proxy {
        client_builder.proxy(proxy)
    } else {
        client_builder.no_proxy()
    };
    let client = client_builder.build()?;

    Ok(client)
}

pub async fn make_backoff_policy(max_timeout_secs: u64) -> ExponentialBackoff {
    let configuration = configuration::CONFIGURATION.read().await;
    let initial_interval = configuration.backoff.initial_interval;
    let multiplier = configuration.backoff.multiplier;
    let max_retry = configuration.backoff.max_retry;
    let wait_times = initial_interval as f32 * (1.0 - multiplier.powi(max_retry as i32 - 1))
        / (1.0 - multiplier);
    let max_timeouts = max_timeout_secs as f32 * max_retry as f32;
    let max_elapsed_time = (wait_times + max_timeouts).ceil() as u64;

    let mut building = ExponentialBackoffBuilder::default();
    let policy = building
        .with_initial_interval(Duration::from_secs(initial_interval))
        .with_multiplier(multiplier)
        .with_randomization_factor(0.2)
        .with_max_elapsed_time(Some(Duration::from_secs(max_elapsed_time)));
    policy.build()
}
