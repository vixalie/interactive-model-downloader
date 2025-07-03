use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail};
use reqwest::{Client, Url};

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
    skip_community: bool,
) -> Result<()> {
    println!("Fetching model metadata...");
    let model_meta = meta::fetch_model_metadata(client, model_id).await?;
    let selected_version = selections::select_model_version(&model_meta, version_id)
        .context("Unable to confirm model version")?;

    println!("Fetching specified version metadata...");
    let selected_version_meta = meta::fetch_model_version_meta(client, selected_version)
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
    let version_file_hash = |id: u64| -> Option<String> {
        version_files
            .iter()
            .find(|f| f.id() == id)
            .map(ModelVersionFile::blake3_hash)
            .flatten()
    };

    for file_id in selected_version_file_ids {
        // 检查缓存数据库中是否已经存在该模型的下载记录，对比数据库中记录的文件位置列表
        // 未下载过的和未使用renew命令的文件将会直接重新下载。
        if let Some(hash) = version_file_hash(file_id) {
            // 只有在存在有效hash数据的时候才进行判断
            let file_locations = cache_db::retreive_civitai_model_locations_by_blake3(&hash);
            if let Ok(Some(locations)) = file_locations {
                let first_exists_location = locations.iter().find(|loc| loc.exists());
                if let Some(file_path) = first_exists_location
                    && !selections::decide_proceeding_or_not(file_path)
                {
                    continue;
                }
            }
        }

        // 下载指定的文件
        println!("Downloading file(s)...");
        let file_name = version_file_name(file_id)
            .with_context(|| format!("Failed to confirm model version file {file_id} name"))?;
        let model_file_name = download_task::download_single_model_file(
            client,
            &selected_version_meta,
            file_id,
            destination_path.as_deref(),
        )
        .await
        .with_context(|| format!("Failed to download model file {file_name}"))?;
        if file_id == primary_file_id {
            target_meta_filename = model_file_name;
        }
    }

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

    let community_images = if !skip_community {
        println!("Fetching community posted images metadata related to model...");
        meta::fetch_model_community_images(client, model_id)
            .await
            .with_context(|| {
                format!("Failed to fetch community posted images coorespond to model {model_id}")
            })?
    } else {
        println!("Skip retreiving community images metadata related to model.");
        Vec::new()
    };

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

pub async fn complete_file_meta<P>(
    client: &Client,
    source_file: P,
    skip_community: bool,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let source_file_path = source_file.as_ref();
    let source_file_path = if let Some(parent) = source_file_path.parent()
        && parent.to_string_lossy().is_empty()
    {
        let parent_dir = env::current_dir().context("Unable to get current working directory")?;
        parent_dir.join(source_file_path)
    } else {
        source_file_path.to_path_buf()
    };
    let working_dir = source_file_path.parent().map(Path::to_path_buf).unwrap();
    if !working_dir.exists() || !working_dir.is_dir() {
        bail!("Source file path is not a valid directory");
    }

    println!("Start to calculate file hash...");
    let source_file_hash = meta::blake3_hash(&source_file_path).context("Calculate file hash")?;
    println!("File hash: {}", source_file_hash.to_ascii_uppercase());

    println!("Save file hash...");
    meta::save_version_file_hash(&source_file_path, &source_file_hash)
        .await
        .context("Save file hash")?;

    println!("Request model version metadata...");
    let model_version_meta =
        match meta::fetch_model_version_meta_by_blake3(client, &source_file_hash).await {
            Ok(meta) => meta,
            Err(e) => {
                return Err(e);
            }
        };

    println!("Collecting related model metadata...");
    let model_meta = meta::fetch_model_metadata(client, model_version_meta.model_id())
        .await
        .context("Request for model metadata")?;
    let source_file_name = source_file_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    println!("Download cover image...");
    let cover_image_file_name = download_task::download_model_version_cover_image(
        client,
        &model_version_meta,
        download_task::ModelVersionFileNamePresent::FileName(source_file_name.clone()),
        Some(&working_dir),
    )
    .await
    .inspect_err(|e| println!("Model version cover download failed: {e}"))
    .ok()
    .flatten();

    let related_community_images = if !skip_community {
        println!("Collecting related community images metadata...");
        meta::fetch_model_community_images(client, model_meta.id())
            .await
            .inspect_err(|e| println!("Community images metadata retreive failed: {e}"))
            .ok()
            .unwrap_or(Vec::new())
    } else {
        println!("Skip collect related community images metadata.");
        Vec::new()
    };

    println!("Save model version readme file...");
    meta::save_model_version_readme(
        &model_meta,
        &model_version_meta,
        &related_community_images,
        cover_image_file_name,
        Some(&working_dir),
        source_file_name,
    )
    .await
    .context("Failed to save model version readme file")?;

    Ok(())
}
