use std::path::PathBuf;

use anyhow::{anyhow, bail};
use reqwest::Url;

mod download_task;
mod meta;
mod model;
mod selections;

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

pub async fn download_from_civitai(
    client: &reqwest::Client,
    model_id: u64,
    version_id: Option<u64>,
    destination_path: Option<&PathBuf>,
) -> anyhow::Result<()> {
    println!("Fetching model metadata...");
    let model_meta = meta::fetch_model_metadata(client, &model_id.to_string()).await?;
    meta::save_model_meta(&model_meta)
        .await
        .map_err(|e| anyhow!("Failed to save model metadata to cache, {}", e))?;
    let selected_version = selections::select_model_version(&model_meta, version_id)
        .map_err(|e| anyhow!("Failed to confirm model version. {}", e))?;
    let selected_version_file_ids = selections::select_model_version_files(&selected_version)
        .map_err(|e| anyhow!("Failed to comfirm model version files. {}", e))?;

    for file_id in selected_version_file_ids {
        println!("Downloading file(s)...");
        download_task::download_single_model_file(
            client,
            &selected_version,
            file_id,
            destination_path.as_ref(),
        )
        .await
        .map_err(|e| anyhow!("Failed to download model file. {}", e))?;
    }

    meta::save_model_version_readme(&model_meta, selected_version.id, destination_path.as_ref())
        .await
        .map_err(|e| anyhow!("Failed to save model version description file. {}", e))?;

    Ok(())
}
