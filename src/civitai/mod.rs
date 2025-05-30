use anyhow::{anyhow, bail};
use reqwest::Url;

mod meta;
mod model;

pub fn try_parse_civitai_model_url(url: &Url) -> anyhow::Result<(String, Option<String>)> {
    let path_segments = url.path_segments();
    let model_id = if let Some(mut segments) = path_segments {
        segments
            .clone()
            .position(|s| s.eq_ignore_ascii_case("models"))
            .and_then(|index| segments.nth(index + 1))
            .map(ToString::to_string)
            .ok_or(anyhow!("The given url does not contain any model id."))?
    } else {
        bail!("The given url does not contain any model id.");
    };

    let model_version_id = url
        .query_pairs()
        .find(|(key, _)| key.eq_ignore_ascii_case("modelVersionId"))
        .map(|(_, value)| value.to_string());

    Ok((model_id, model_version_id))
}
