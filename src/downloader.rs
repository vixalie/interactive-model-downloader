use hyper::{HeaderMap, header};
use reqwest::{ClientBuilder, Url};

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

pub fn make_client(platform: &Platform) -> anyhow::Result<Client> {
    let config = crate::configuration::CONFIGURATION.lock().unwrap();
    let proxy = config.proxy.get_proxy();

    let mut default_headers = HeaderMap::new();
    default_headers.insert(
        header::AUTHORIZATION,
        header::HeaderValue::from_str(&format!(
            "Bearer {}",
            match platform {
                Platform::Civitai => config.civitai.api_key.clone(),
                Platform::HuggingFace => config.huggingface.api_key.clone(),
            }
            .unwrap_or_default()
        )),
    );

    let client_builder = ClientBuilder::new();
    client_builder.user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36");
    if let Some(proxy) = proxy {
        client_builder.proxy(proxy);
    } else {
        client_builder.no_proxy();
    }
    client_builder.default_headers(default_headers);

    Ok(client_builder.build())
}
