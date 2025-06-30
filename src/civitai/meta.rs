use std::{
    env,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Ok, Result, anyhow, bail};
use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use reqwest::{Client, Method, header};
use serde_json::Value;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::cache_db;

use super::model::{self, ImageMeta};

const FILENAME_SET: &AsciiSet = &NON_ALPHANUMERIC.remove(b'.').remove(b'_').remove(b'-');

pub async fn fetch_model_metadata(client: &Client, model_id: u64) -> Result<model::Model> {
    let config = crate::configuration::CONFIGURATION.read().await;
    let model_meta_url = format!("https://civitai.com/api/v1/models/{model_id}");
    let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
    let meta_request_builder = client
        .request(Method::GET, model_meta_url)
        .bearer_auth(civitai_auth_key)
        .header(header::ACCEPT, "application/json");
    let request = meta_request_builder.build()?;

    let meta_response = client
        .execute(request)
        .await
        .context("Failed to retreive model meta info")?;
    let raw_content = meta_response
        .bytes()
        .await
        .context("Failed to retreive model meta info")?;
    let content = String::from_utf8_lossy(&raw_content);

    let raw_model_meta =
        serde_json::from_str::<Value>(&content).context("Failed to parse model meta info")?;
    let model_meta = model::Model::try_from(&raw_model_meta)?;

    cache_db::store_civitai_model(&model_meta)?;

    Ok(model_meta)
}

pub async fn fetch_model_version_meta(
    client: &Client,
    version_id: u64,
) -> Result<model::ModelVersion> {
    let config = crate::configuration::CONFIGURATION.read().await;
    let model_meta_url = format!("https://civitai.com/api/v1/model-versions/{version_id}");
    let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
    let meta_request_builder = client
        .request(Method::GET, model_meta_url)
        .bearer_auth(civitai_auth_key)
        .header(header::ACCEPT, "application/json");
    let request = meta_request_builder.build()?;

    let meta_response = client
        .execute(request)
        .await
        .context("Failed to retreive model version meta info")?;
    let raw_content = meta_response
        .bytes()
        .await
        .context("Failed to retreive model version meta info")?;
    let content = String::from_utf8_lossy(&raw_content);

    let raw_model_version_meta = serde_json::from_str::<Value>(&content)
        .context("Failed to parse model version meta info")?;
    let model_version_meta = model::ModelVersion::try_from(&raw_model_version_meta)?;

    cache_db::store_civitai_model_version(&model_version_meta)?;

    Ok(model_version_meta)
}

#[allow(dead_code)]
pub async fn fetch_model_version_meta_by_blake3(
    client: &Client,
    model_hash: &str,
) -> Result<model::ModelVersion> {
    let config = crate::configuration::CONFIGURATION.read().await;
    let model_meta_url = format!("https://civitai.com/api/v1/model-versions/by-hash/{model_hash}");
    let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
    let meta_request_builder = client
        .request(Method::GET, model_meta_url)
        .bearer_auth(civitai_auth_key)
        .header(header::ACCEPT, "application/json");
    let request = meta_request_builder.build()?;

    let meta_response = client
        .execute(request)
        .await
        .context("Failed to retreive model version meta info")?;
    let raw_content = meta_response
        .bytes()
        .await
        .context("Failed to retreive model version meta info")?;
    let content = String::from_utf8_lossy(&raw_content);

    let raw_model_version_meta = serde_json::from_str::<Value>(&content)
        .context("Failed to parse model version meta info")?;
    if let Some(err_field) = raw_model_version_meta.get("error") {
        bail!(
            "Civitai.com returns error: {}",
            err_field.as_str().unwrap_or_default()
        );
    }
    let model_version_meta = model::ModelVersion::try_from(&raw_model_version_meta)?;

    cache_db::store_civitai_model_version(&model_version_meta)?;

    Ok(model_version_meta)
}

pub async fn fetch_model_community_images(
    client: &Client,
    model_id: u64,
) -> Result<Vec<model::ModelCommunityImage>> {
    let config = crate::configuration::CONFIGURATION.read().await;
    let model_meta_url = format!("https://civitai.com/api/v1/images");
    let civitai_auth_key = config.civitai.api_key.clone().unwrap_or_default();
    let meta_request_builder = client
        .request(Method::GET, model_meta_url)
        .bearer_auth(civitai_auth_key)
        .header(header::ACCEPT, "application/json")
        .query(&[("modelId", model_id), ("limit", 50)])
        .timeout(Duration::from_secs(45));
    let request = meta_request_builder.build()?;

    let meta_response = client.execute(request).await;
    if meta_response.is_err() {
        println!(
            "Failed to retreive community images metadata, maybe timeout, skip community images collection."
        );
        return Ok(vec![]);
    }
    let meta_response = meta_response.unwrap();
    let raw_content = meta_response
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to retreive model meta info: {}", e.to_string()))
        .context("Request for community images")?;
    let content = String::from_utf8_lossy(&raw_content);

    let raw_response_value = serde_json::from_str::<Value>(&content);
    if raw_response_value.is_err() {
        println!(
            "Failed to retreive community images metadata, cancel community images collection.\nCancel community images collection."
        );
        return Ok(Vec::new());
    }
    let raw_response_value = raw_response_value.unwrap();
    let err_field = raw_response_value.get("error");
    if let Some(err_field) = err_field {
        println!(
            "Civitai.com returns error: {}\nCancel community images collection.",
            err_field.as_str().unwrap_or_default()
        );
        return Ok(Vec::new());
    }
    let response_items = raw_response_value.get("items");
    if response_items.is_none() {
        println!(
            "Retreived community images response is missing required field - [items]\nCancel community images collection."
        );
        return Ok(Vec::new());
    }
    let response_items = response_items.unwrap();
    if !response_items.is_array() {
        println!(
            "Retreived community images response is not valid.\nCancel community images collection."
        );
        return Ok(Vec::new());
    }

    let mut model_community_images = Vec::new();
    let items = response_items.as_array().unwrap();
    for item in items {
        let image = model::ModelCommunityImage::try_from(item).context("Parse community image")?;
        model_community_images.push(image);
    }

    Ok(model_community_images)
}

async fn write_image_meta(file: &mut File, image: &dyn ImageMeta) -> Result<()> {
    let posi_prompt = image.positive_prompt();
    if !posi_prompt.is_some() {
        bail!("No valid positive prompt");
    }
    file.write_all(b"===\n\n").await?;

    let image_url = image.url();
    let encoed_url = utf8_percent_encode(&image_url, &FILENAME_SET).to_string();
    file.write_all(format!("[Click to view sample image]({})\n\n", encoed_url).as_bytes())
        .await?;

    if let Some(prompt) = posi_prompt {
        file.write_all(format!("**Positive Prompt:**\n\n{prompt}\n\n").as_bytes())
            .await?;
    }
    if let Some(neg_prompt) = image.negative_prompt() {
        file.write_all(format!("**Negative Prompt:**\n\n{neg_prompt}\n\n").as_bytes())
            .await?;
    }
    if let Some(sampler) = image.sampler() {
        file.write_all(format!("**Sampler:** {sampler}\n\n").as_bytes())
            .await?;
    }
    if let Some(scheduler) = image.scheduler() {
        file.write_all(format!("**Scheduler:** {scheduler}\n\n").as_bytes())
            .await?;
    }
    if let Some(seed) = image.seed() {
        file.write_all(format!("**Seed:** {seed}\n\n").as_bytes())
            .await?;
    }
    if let Some(steps) = image.steps() {
        file.write_all(format!("**Steps:** {steps}\n\n").as_bytes())
            .await?;
    }
    if let Some(cfg_scale) = image.cfg_scale() {
        file.write_all(format!("**CFG Scale:** {cfg_scale:.2}\n\n").as_bytes())
            .await?;
    }
    file.write_all(b"===\n\n").await?;

    Ok(())
}

pub async fn save_model_version_readme(
    model: &model::Model,
    model_version: &model::ModelVersion,
    community_images: &[model::ModelCommunityImage],
    cover_image_filename: Option<String>,
    destination_path: Option<&PathBuf>,
    meta_filename: String,
) -> Result<()> {
    let target_dir = match destination_path {
        Some(path) => path.clone(),
        None => std::env::current_dir()?,
    };
    let filename = PathBuf::from(meta_filename);
    let basename = filename.file_stem().unwrap_or_default();
    let meta_file_path = target_dir.join(format!("{}.md", basename.to_string_lossy()));

    let model_description = model.markdown_description();
    let model_version_description = model_version.markdown_description();

    let mut meta_file = File::create(meta_file_path).await?;
    meta_file
        .write_all(format!("# {}\n\n", model.name()).as_bytes())
        .await?;
    meta_file.write_all(model_description.as_bytes()).await?;
    meta_file
        .write_all(format!("\n\n## Version: {}\n\n", model_version.name()).as_bytes())
        .await?;

    if let Some(image) = cover_image_filename {
        let encoded_file_path = utf8_percent_encode(&image, &FILENAME_SET).to_string();
        meta_file
            .write_all(format!("![](./{encoded_file_path})\n\n").as_bytes())
            .await?;
    }

    if let Some(description) = model_version_description {
        meta_file.write_all(description.as_bytes()).await?;
    }
    meta_file.write_all(b"\n\n").await?;

    let trained_words = model_version.trained_words();
    if trained_words.len() > 0 {
        meta_file.write_all(b"## Trained Words\n\n").await?;
        for word in trained_words.iter() {
            meta_file
                .write_all(format!("- {word}\n").as_bytes())
                .await?;
        }
    }

    let version_cover_images = model_version.images()?;
    if !version_cover_images.is_empty() {
        meta_file.write_all(b"## Cover image prompts\n\n").await?;
        for image in version_cover_images {
            if image.positive_prompt().is_some() {
                write_image_meta(&mut meta_file, &image).await?;
            }
        }
    }

    if !community_images.is_empty() {
        meta_file
            .write_all(b"## Community image prompts\n\n")
            .await?;
        for image in community_images {
            if image.positive_prompt().is_some() {
                write_image_meta(&mut meta_file, image).await?;
            }
        }
    }

    meta_file.flush().await?;

    Ok(())
}

#[allow(dead_code)]
pub fn blake3_hash<P: AsRef<Path>>(target_file: P) -> Result<String> {
    let target_file_path = target_file.as_ref();
    if !target_file_path.exists() {
        bail!("Request file {} not exists", target_file_path.display());
    }

    let mut file = std::fs::File::open(target_file_path)?;
    let mut reader = BufReader::new(&mut file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 512 * 1024];

    loop {
        let read_size = reader.read(&mut buffer)?;
        if read_size == 0 {
            break;
        }
        hasher.update(&buffer[0..read_size]);
    }
    let hash = hasher.finalize();
    let hash_str = hash.to_hex().to_string().to_uppercase();

    Ok(hash_str)
}

pub async fn save_version_file_hash<P: AsRef<Path>>(source_file_path: P, hash: &str) -> Result<()> {
    let source_file = source_file_path.as_ref();

    let model_file_name = source_file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap();
    let hash_file_name = format!("{model_file_name}.blake3");
    let hash_file_path = match source_file.parent() {
        Some(dir) => dir.to_path_buf(),
        None => env::current_dir()?,
    }
    .join(hash_file_name);

    let mut hash_file = File::create(hash_file_path).await?;
    let blake3_str = hash.to_string().to_uppercase();
    hash_file.write_all(blake3_str.as_bytes()).await?;
    hash_file.flush().await?;

    Ok(())
}
