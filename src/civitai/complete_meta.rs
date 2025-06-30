use std::path::Path;

use anyhow::{Context, Result, bail};
use reqwest::Client;

use crate::civitai::{
    download_task,
    meta::{
        self, blake3_hash, fetch_model_community_images, fetch_model_metadata,
        fetch_model_version_meta_by_blake3,
    },
};

#[allow(dead_code)]
pub async fn complete_file_meta<P>(client: &Client, source_file: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let source_file_path = source_file.as_ref();
    let working_dir = source_file_path.parent().map(Path::to_path_buf).unwrap();
    if !working_dir.exists() || !working_dir.is_dir() {
        bail!("Source file path is not a valid directory");
    }

    print!("Start to calculate file hash...");
    let source_file_hash = blake3_hash(source_file_path)?;
    println!("OK\nFile hash: {}", source_file_hash.to_ascii_uppercase());

    print!("Request model version metadata...");
    let model_version_meta = fetch_model_version_meta_by_blake3(client, &source_file_hash).await?;
    println!("OK");

    println!("Collecting related model metadata...");
    let model_meta = fetch_model_metadata(client, model_version_meta.model_id()).await?;
    let source_file_name = source_file_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    print!("Download cover image...");
    let cover_image_file_name = download_task::download_model_version_cover_image(
        client,
        &model_version_meta,
        download_task::ModelVersionFileNamePresent::FileName(source_file_name.clone()),
        Some(&working_dir),
    )
    .await
    .context("Failed to download cover image")?;
    println!("OK");

    print!("Collecting related community images metadata...");
    let related_community_images = fetch_model_community_images(client, model_meta.id()).await?;
    println!("OK");

    print!("Save model version readme file...");
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
    println!("OK");

    Ok(())
}
