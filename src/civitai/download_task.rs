use std::{
    cmp::min,
    env,
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::{Context, anyhow, bail};
use futures_util::StreamExt;
use image::ImageReader;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
};

use crate::{cache_db, civitai::meta::save_version_file_hash};

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
        .find(|f| f.id == file_id)
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
        .request(reqwest::Method::GET, selected_file.download_url.clone())
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
    let mut file = File::create(target_file_path).await?;
    let mut downloaded_size: u64 = 0;
    let mut download_stream = response.bytes_stream();

    while let Some(chunk) = download_stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded_size = min(downloaded_size + chunk.len() as u64, file_legnth);
        pb.set_position(downloaded_size);
    }
    file.flush().await?;

    pb.finish_with_message(format!(
        "File {} download completed.",
        selected_file.name.clone()
    ));

    // Run crc32 check
    let mut check_file = File::open(target_file_path).await?;
    let mut reader = BufReader::new(&mut check_file);
    let mut crc32_hasher = crc32fast::Hasher::new();
    let mut blake3_hasher = blake3::Hasher::new();
    let mut data_buffer = [0u8; 512 * 1024];

    loop {
        let read_size = reader.read(&mut data_buffer).await?;
        if read_size == 0 {
            break;
        }
        crc32_hasher.update(&data_buffer[0..read_size]);
        blake3_hasher.update(&data_buffer[0..read_size]);
    }
    let crc32_checksum = crc32_hasher.finalize();
    let blake3_checksum = blake3_hasher.finalize();

    // Check crc32
    if let Some(control_value_str) = selected_file.crc32() {
        let control_value = u32::from_str_radix(&control_value_str, 16)?;
        if crc32_checksum == control_value {
            println!("CRC32 check passed.");
        } else {
            println!("CRC32 check failed. Maybe need to redownload.");
        }
    }

    // Record model blake3 hash
    let blake3_str = blake3_checksum.to_hex().to_string();
    save_version_file_hash(&target_file_path, &blake3_str)
        .await
        .context("Save file blake3 hash record")?;

    cache_db::store_civitai_model_file_location(
        model_version_meta.model_id(),
        model_version_meta.id(),
        file_id,
        &target_file_path,
        &blake3_str,
    )
    .context("Store file location to cache database")?;

    Ok(selected_file.name.clone())
}

pub async fn download_model_version_cover_image(
    client: &Client,
    version_meta: &model::ModelVersion,
    downloaded_file_id: u64,
    destination_path: Option<&PathBuf>,
) -> anyhow::Result<Option<String>> {
    let version_files = version_meta
        .files()
        .context("Fetch file list in model version")?;
    let downloaded_file_name = version_files
        .iter()
        .find(|f| f.id() == downloaded_file_id)
        .map(model::ModelVersionFile::name)
        .map(PathBuf::from)
        .and_then(|p| p.file_stem())
        .map(|s| s.to_string_lossy().into_owned())
        .ok_or(anyhow!("Metadata of downloaded file is not found"))?;
    let cover_image = version_meta
        .images()?
        .into_iter()
        .find(|img| !img.media_type().eq_ignore_ascii_case("video"));

    if cover_image.is_none() {
        return Ok(None);
    }
    let cover_image = cover_image.unwrap();

    let config = crate::configuration::CONFIGURATION.read().await;
    let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
    let download_request = client
        .request(reqwest::Method::GET, cover_image.url())
        .bearer_auth(civitai_auth_key);
    let request = download_request.build()?;

    let response = client.execute(request).await?;
    let file_legnth = response
        .content_length()
        .ok_or(anyhow!("Incorrect cover image length"))?;

    let image_bytes = response.bytes().await?;
    let image_buffer = Cursor::new(image_bytes);
    let image = ImageReader::new(image_buffer)
        .with_guessed_format()
        .context("Unregconized image format")?
        .decode()
        .context("Unable to decode image")?;

    let preview_image_filename = format!("{downloaded_file_name}.cover.jpg");
    let target_image_path = match destination_path {
        Some(given_path) => given_path.clone(),
        None => env::current_dir()?,
    }
    .join(preview_image_filename);
    image.save_with_format(&target_image_path, image::ImageFormat::Jpeg)?;

    Ok(Some(preview_image_filename))
}
