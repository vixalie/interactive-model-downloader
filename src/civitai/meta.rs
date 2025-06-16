use std::path::PathBuf;

use anyhow::{anyhow, ensure};
use reqwest::{Client, Method, header};
use serde_json::Value;
use tokio::{fs::File, io::AsyncWriteExt};

use super::model;

pub async fn fetch_model_metadata(client: &Client, model_id: &str) -> anyhow::Result<model::Model> {
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
        .map_err(|e| anyhow!("Failed to retreive model meta info: {}", e.to_string()))?;
    let raw_content = meta_response
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to retreive model meta info: {}", e.to_string()))?;
    let content = String::from_utf8_lossy(&raw_content);

    // let model_meta = serde_json::from_str::<model::Model>(&content)
    //     .map_err(|e| anyhow!("Failed to parse model meta info: {}", e.to_string()))?;

    // let model_meta = meta_response
    //     .json::<model::Model>()
    //     .await
    //     .map_err(|e| anyhow!("Failed to decode response: {}", e.to_string()))?;

    let raw_model_meta = serde_json::from_str::<Value>(&content)
        .map_err(|e| anyhow!("Failed to parse model meta info: {}", e.to_string()))?;
    let model_meta = model::Model::try_from(&raw_model_meta)?;

    Ok(model_meta)
}

pub async fn save_model_meta(model_meta: &model::Model) -> anyhow::Result<()> {
    let cache_dir = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .map(|home_dir| home_dir.join(".config").join("imd").join("cache"));
    ensure!(cache_dir.is_some(), "Failed to get config directory.");
    let cache_dir = cache_dir.unwrap();

    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir)?;
    }

    let meta_file_path = cache_dir.join(format!("{}.json", model_meta.id));
    let mut meta_file = File::create(meta_file_path).await?;
    let meta_content = serde_json::to_string_pretty(model_meta)?;
    meta_file.write_all(meta_content.as_bytes()).await?;

    Ok(())
}

pub async fn save_model_version_readme(
    model_meta: &model::Model,
    version_id: u64,
    destination_path: Option<&PathBuf>,
) -> anyhow::Result<()> {
    let target_dir = match destination_path {
        Some(path) => path.clone(),
        None => std::env::current_dir()?,
    };
    let model_version_meta = model_meta
        .model_versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or(anyhow!("The given model version does not exist."))?;
    let model_version_filename = PathBuf::from(model_version_meta.name.clone())
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or(format!("{}", model_version_meta.id));
    let meta_file_path = target_dir.join(format!("{model_version_filename}.md"));

    let model_description = html2md_rs::to_md::safe_from_html_to_md(model_meta.description.clone())
        .map_err(|e| anyhow!("Failed to convert model description to markdown, {}", e))?;
    let model_version_description = html2md_rs::to_md::safe_from_html_to_md(
        model_version_meta
            .description
            .clone()
            .unwrap_or("".to_string()),
    )
    .map_err(|e| {
        anyhow!(
            "Failed to convert model version description to markdown, {}",
            e
        )
    })?;

    let mut meta_file = File::create(meta_file_path).await?;
    meta_file
        .write_all(format!("# {}\n\n", model_meta.name).as_bytes())
        .await?;
    meta_file.write_all(model_description.as_bytes()).await?;
    meta_file
        .write_all(format!("\n\n## Version: {}\n\n", model_version_meta.name).as_bytes())
        .await?;
    meta_file
        .write_all(model_version_description.as_bytes())
        .await?;
    meta_file.write_all(b"\n\n").await?;

    if model_version_meta.trained_words.len() > 0 {
        meta_file.write_all(b"## Trained Words\n\n").await?;
        for word in model_version_meta.trained_words.iter() {
            meta_file
                .write_all(format!("- {}\n", word).as_bytes())
                .await?;
        }
    }

    meta_file.flush().await?;

    Ok(())
}
