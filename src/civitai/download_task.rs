use std::{cmp::min, env, path::PathBuf};

use anyhow::anyhow;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use tokio::{fs::File, io::AsyncWriteExt};

use super::model;

pub async fn download_single_model_file(
    client: &Client,
    model_version_meta: &model::ModelVersion,
    file_id: u64,
    destination_path: Option<&PathBuf>,
) -> anyhow::Result<String> {
    let selected_file = model_version_meta
        .files()
        .iter()
        .find(|f| f.id == file_id)
        .context("Request model file is not found.")?;
    println!("Downloading file: {}", selected_file.name);
    let target_file_path = match destination_path {
        Some(given_path) => given_path.clone(),
        None => env::current_dir()?,
    }
    .join(selected_file.name.clone());
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

    Ok(selected_file.name.clone())
}
