use anyhow::anyhow;
use reqwest::{Client, Method, Request, header};

use super::model;

pub async fn fetch_model_metadata(client: &Client, model_id: &str) -> anyhow::Result<model::Model> {
    let config = crate::configuration::CONFIGURATION
        .lock()
        .map_err(|_| anyhow!("Failed to retreive configuration."))?;
    let model_meta_url = format!("https://civitai.com/api/v1/models/{model_id}");
    let meta_request_builder = client.request(Method::GET, model_meta_url);
    meta_request_builder.bearer_auth(&config.civitai.api_key);
    meta_request_builder.header(header::ACCEPT, "application/json");
    let request = meta_request_builder.build()?;

    let meta_response = client
        .execute(request)
        .await
        .map_err(|e| anyhow!("Failed to retreive model meta info: {}", e.to_string()))?;

    let model_meta = meta_response.json::<model::Model>().await?;

    Ok(model_meta)
}

pub async fn save_model_meta(model_meta: &model::Model) -> anyhow::Result<()> {
    let current_dir = std::env::current_dir()?;
    let meta_file_path = current_dir.join(format!("{}.json", model_meta.id));
    let meta_file = File::create(meta_file_path)?;
    serde_json::to_writer_pretty(meta_file, model_meta)?;
}
