use reqwest::Url;

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
