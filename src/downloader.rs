use reqwest::{Client, ClientBuilder, Url};

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
