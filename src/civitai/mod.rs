use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, bail};
use reqwest::Url;

mod complete_meta;
mod download_task;
mod meta;
mod model;
mod selections;

pub use model::*;

use crate::cache_db;

pub fn try_parse_civitai_model_url(url: &Url) -> Result<(String, Option<String>)> {
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
) -> Result<()> {
    println!("Fetching model metadata...");
    let model_meta = meta::fetch_model_metadata(client, &model_id.to_string()).await?;
    let selected_version = selections::select_model_version(&model_meta, version_id)
        .context("Unable to confirm model version")?;

    println!("Fetching specified version metadata...");
    let selected_version_meta = meta::fetch_model_version_meta(client, selected_version.id())
        .await
        .with_context(|| format!("Failed to fetch version {selected_version} detail metadata"))?;

    let selected_version_file_ids = selections::select_model_version_files(&selected_version_meta)
        .context("Failed to confirm model version files")?;

    let version_files = selected_version_meta.files()?;
    let primary_file_id = version_files
        .iter()
        .find(|f| f.is_primary().unwrap_or_default())
        .map(|f| f.id())
        .unwrap_or_else(|| version_files[0].id());
    let mut target_meta_filename = String::new();

    let version_file_name = |id: u64| -> Option<String> {
        version_files
            .iter()
            .find(|f| f.id() == id)
            .map(ModelVersionFile::name)
    };

    for file_id in selected_version_file_ids {
        // todo: 检查缓存数据库中是否已经存在该模型的下载记录，对比数据库中记录的文件位置列表
        println!("Downloading file(s)...");
        let file_name = version_file_name(file_id)
            .with_context(|| format!("Failed to confirm model version file {file_id} name"))?;
        let model_file_name = download_task::download_single_model_file(
            client,
            &selected_version,
            file_id,
            destination_path.as_deref(),
        )
        .await
        .with_context(|| format!("Failed to download model file {file_name}"))?;
        if file_id == primary_file_id {
            target_meta_filename = model_file_name;
        }
    }

    println!("Fetching community posted images metadata cooresponding to model...");
    let community_images = meta::fetch_model_community_images(client, model_id)
        .await
        .with_context(|| {
            format!("Failed to fetch community posted images coorespond to model {model_id}")
        })?;

    let cover_image_filename = download_task::download_model_version_cover_image(
        client,
        &selected_version_meta,
        download_task::ModelVersionFileNamePresent::FileID(primary_file_id),
        destination_path.as_deref(),
    )
    .await
    .with_context(|| {
        format!("Failed to download cover image for model version {selected_version}")
    })?;

    meta::save_model_version_readme(
        &model_meta,
        &selected_version_meta,
        &community_images,
        cover_image_filename,
        destination_path.as_deref(),
        target_meta_filename,
    )
    .await
    .context("Failed to save model version description file")?;

    Ok(())
}
