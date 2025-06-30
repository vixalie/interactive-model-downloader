use std::{cmp::min, env, io::Cursor, path::PathBuf};

use anyhow::{Context, anyhow};
use futures_util::StreamExt;
use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    cache_db,
    civitai::{
        ImageMeta,
        meta::{self, save_version_file_hash},
    },
    downloader::make_backoff_policy,
    utils::duration_to_sec_string,
};

use super::model;

pub async fn download_single_model_file(
    client: &Client,
    model_version_meta: &model::ModelVersion,
    file_id: u64,
    destination_path: Option<&PathBuf>,
) -> anyhow::Result<String> {
    let selected_file = model_version_meta
        .files()?
        .into_iter()
        .find(|f| f.id() == file_id)
        .ok_or(anyhow!("Request model file is not found"))?;
    println!("Downloading file: {}", selected_file.name());
    let target_file_path = match destination_path {
        Some(given_path) => given_path.clone(),
        None => env::current_dir()?,
    }
    .join(selected_file.name());
    let config = crate::configuration::CONFIGURATION.read().await;
    let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
    let download_request = client
        .request(reqwest::Method::GET, selected_file.download_url())
        .bearer_auth(civitai_auth_key);
    let request = download_request.build()?;

    let response = client.execute(request).await?;

    let file_legnth = response
        .content_length()
        .ok_or(anyhow!("Incorrect model file length"))?;

    let pb = ProgressBar::new(file_legnth);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{wide_bar:.cyan/blue}] {decimal_bytes}/{decimal_total_bytes} [{elapsed}] ETA:{eta}")?
            .progress_chars("=>-"),
    );
    let mut file = File::create(&target_file_path).await?;
    let mut downloaded_size: u64 = 0;
    let mut download_stream = response.bytes_stream();

    while let Some(chunk) = download_stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded_size = min(downloaded_size + chunk.len() as u64, file_legnth);
        pb.set_position(downloaded_size);
    }
    file.flush().await?;

    pb.finish_with_message(format!("File {} download completed.", selected_file.name()));

    // Run blake3 check
    let blake3_checksum = meta::blake3_hash(&target_file_path)?;

    // Check crc32
    if selected_file.match_by_blake3(&blake3_checksum) {
        println!("File blake3 check passed.");
    } else {
        println!("File blake3 check failed. Maybe need to redownload.");
    }

    // Record model blake3 hash
    save_version_file_hash(&target_file_path, &blake3_checksum)
        .await
        .context("Save file blake3 hash record")?;

    cache_db::store_civitai_model_file_location(
        model_version_meta.model_id(),
        model_version_meta.id(),
        file_id,
        &blake3_checksum,
        &target_file_path,
    )
    .context("Store file location to cache database")?;

    Ok(selected_file.name())
}

#[allow(dead_code)]
pub enum ModelVersionFileNamePresent {
    FileID(u64),
    FileName(String),
    PrimaryFile,
}

pub async fn download_model_version_cover_image(
    client: &Client,
    version_meta: &model::ModelVersion,
    file_present: ModelVersionFileNamePresent,
    destination_path: Option<&PathBuf>,
) -> anyhow::Result<Option<String>> {
    let file_name = match file_present {
        ModelVersionFileNamePresent::FileID(file_id) => {
            let version_files = version_meta
                .files()
                .context("Fetch file list in model version")?;
            version_files
                .iter()
                .find(|f| f.id() == file_id)
                .map(model::ModelVersionFile::name)
        }
        ModelVersionFileNamePresent::FileName(file_name) => {
            let file_path = PathBuf::from(file_name);
            file_path
                .file_name()
                .map(|p| p.to_string_lossy().into_owned())
        }
        ModelVersionFileNamePresent::PrimaryFile => {
            let version_files = version_meta
                .files()
                .context("Fetch file list in model version")?;
            version_files
                .iter()
                .find(|f| f.is_primary().unwrap_or_default())
                .map(model::ModelVersionFile::name)
        }
    };

    let downloaded_file_name = file_name
        .map(PathBuf::from)
        .and_then(|p| p.file_stem().map(|fs| fs.to_string_lossy().into_owned()))
        .ok_or(anyhow!("Metadata of downloaded file is not found"))?;
    let cover_image = version_meta
        .images()?
        .into_iter()
        .find(|img| !img.media_type().eq_ignore_ascii_case("video"));

    if cover_image.is_none() {
        return Ok(None);
    }
    let cover_image = cover_image.unwrap();

    let task = async || {
        println!("Try to fetch cover image.");
        let config = crate::configuration::CONFIGURATION.read().await;
        let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
        let download_request = client
            .request(reqwest::Method::GET, cover_image.url())
            .bearer_auth(civitai_auth_key);
        let request = download_request.build().map_err(|e| {
            backoff::Error::transient(anyhow!("Failed to build cover image download request: {e}"))
        })?;

        let response = client.execute(request).await.map_err(|e| {
            backoff::Error::transient(anyhow!(
                "Failed to execute cover image download request: {e}"
            ))
        })?;
        let image_bytes = response.bytes().await.map_err(|e| {
            backoff::Error::transient(anyhow!("Failed to read cover image content: {e}"))
        })?;

        Ok(image_bytes)
    };
    let notify_op = |_: anyhow::Error, d| {
        println!(
            "Failed to download cover image, will try again {} later.",
            duration_to_sec_string(&d)
        );
    };
    let policy = make_backoff_policy(300);
    let image_bytes = backoff::future::retry_notify(policy, task, notify_op)
        .await
        .context("Download cover image")?;

    let image_buffer = Cursor::new(image_bytes);
    let image = ImageReader::new(image_buffer)
        .with_guessed_format()
        .context("Unregconized image format")?
        .decode()
        .context("Unable to decode image")?;

    let old_preview_image_filename = format!("{downloaded_file_name}.cover.jpg");
    let clear_path = match destination_path {
        Some(given_path) => given_path.clone(),
        None => env::current_dir()?,
    }
    .join(&old_preview_image_filename);
    if clear_path.exists() && clear_path.is_file() {
        tokio::fs::remove_file(clear_path).await?;
    }

    let preview_image_filename = format!("{downloaded_file_name}.cover.png");
    let target_image_path = match destination_path {
        Some(given_path) => given_path.clone(),
        None => env::current_dir()?,
    }
    .join(&preview_image_filename);
    image.save_with_format(&target_image_path, image::ImageFormat::Png)?;

    Ok(Some(preview_image_filename))
}
