use std::path::PathBuf;

use anyhow::{Context, Result, anyhow, ensure};
use reqwest::{Client, Method, header};
use serde_json::Value;
use tokio::{fs::File, io::AsyncWriteExt};

use crate::cache_db;

use super::model::{self, ImageMeta};

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

    cache_db::store_civitai_model_meta(&model_meta)?;

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

    cache_db::store_civitai_model_version_meta(&model_version_meta)?;

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
        .query(&[("modelId", model_id)]);
    let request = meta_request_builder.build()?;

    let meta_response = client
        .execute(request)
        .await
        .map_err(|e| anyhow!("Failed to retreive model meta info: {}", e.to_string()))?;
    let raw_content = meta_response
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to retreive model meta info: {}", e.to_string()))?;
    let content = String::from_utf8_lossy(&raw_content);

    let raw_response_value = serde_json::from_str::<Value>(&content).map_err(|e| {
        anyhow!(
            "Failed to parse model community images info: {}",
            e.to_string()
        )
    })?;
    let response_items = raw_response_value.get("items").ok_or(anyhow!(
        "Failed to parse model community images info: items not found"
    ))?;
    ensure!(
        response_items.is_array(),
        "Retreived community images response is not valid"
    );

    let mut model_community_images = Vec::new();
    for item in response_items.as_array()? {
        let image = model::ModelCommunityImage::try_from(item)?;
        model_community_images.push(image);
        cache_db::store_civitai_model_community_image(model_id, &image)?;
    }

    Ok(model_community_images)
}

async fn write_image_meta(file: &mut File, image: &dyn ImageMeta) -> Result<()> {
    let posi_prompt = image.positive_prompt();
    ensure!(posi_prompt.is_some(), "No valid positive prompt");
    file.write_all("===\n\n").await?;
    if let Some(prompt) = posi_prompt {
        file.write_all(format!("**Positive Prompt:**\n\n{prompt}\n\n"))
            .await?;
    }
    if let Some(neg_prompt) = image.negative_prompt() {
        file.write_all(format!("**Negative Prompt:**\n\n{neg_prompt}\n\n"))
            .await?;
    }
    if let Some(sampler) = image.sampler() {
        file.write_all(format!("**Sampler:** {sampler}\n\n"))
            .await?;
    }
    if let Some(scheduler) = image.scheduler() {
        file.write_all(format!("**Scheduler:** {scheduler}\n\n"))
            .await?;
    }
    if let Some(seed) = image.seed() {
        file.write_all(format!("**Seed:** {seed}\n\n")).await?;
    }
    if let Some(steps) = image.steps() {
        file.write_all(format!("**Steps:** {steps}\n\n")).await?;
    }
    if let Some(cfg_scale) = image.cfg_scale() {
        file.write_all(format!("**CFG Scale:** {cfg_scale:.2}\n\n"))
            .await?;
    }
    file.write_all("===\n\n").await?;

    Ok(())
}

pub async fn save_model_version_readme(
    model: &model::Model,
    model_version: &model::ModelVersion,
    community_images: &[model::ModelCommunityImage],
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
        .write_all(format!("# {}\n\n", model.name()))
        .await?;
    meta_file.write_all(model_description.as_bytes()).await?;
    meta_file
        .write_all(format!("\n\n## Version: {}\n\n", model_version_meta.name))
        .await?;
    if let Some(description) = model_version_description {
        meta_file.write_all(description).await?;
    }
    meta_file.write_all(b"\n\n").await?;

    if model_version_meta.trained_words.len() > 0 {
        meta_file.write_all(b"## Trained Words\n\n").await?;
        for word in model_version_meta.trained_words.iter() {
            meta_file.write_all(format!("- {word}\n")).await?;
        }
    }

    let version_cover_images = model_version.images()?;
    if !version_cover_images.is_empty() {
        meta_file.write_all("## Cover image prompts\n\n").await?;
        for image in version_cover_images {
            write_image_meta(&mut meta_file, &image).await?;
        }
    }

    if !community_images.is_empty() {
        meta_file
            .write_all("## Community image prompts\n\n")
            .await?;
        for image in community_images {
            write_image_meta(&mut meta_file, image).await?;
        }
    }

    meta_file.flush().await?;

    Ok(())
}
