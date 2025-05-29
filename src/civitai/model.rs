use serde::{Deserialize, Serialize};
use time::UtcDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    Checkpoint,
    TextualInversion,
    Hypernetwork,
    AestheticGradient,
    LORA,
    Controlnet,
    Poses,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelMode {
    Archived,
    TakeDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileScanResult {
    Pending,
    Success,
    Danger,
    Error,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub username: String,
    pub image: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStats {
    pub download_count: u64,
    pub favorite_coung: u64,
    pub comment_coung: u64,
    pub rating_count: u64,
    pub rating: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelFileFloatingPoints {
    FP16,
    FP32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelSize {
    Full,
    Pruned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelFileFormat {
    SafeTensor,
    PickleTensor,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionFileMeta {
    pub fp: Option<ModelFileFloatingPoints>,
    pub size: Option<ModelSize>,
    pub format: Option<ModelFileFormat>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionFile {
    pub size_kb: f64,
    pub pickle_scan_result: FileScanResult,
    pub virus_scan_result: FileScanResult,
    #[serde(default)]
    pub scanned_at: Option<UtcDateTime>,
    pub primary: Option<bool>,
    pub metadata: ModelVersionFileMeta,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionImage {
    pub id: String,
    pub url: String,
    pub nsfw: String,
    pub width: u32,
    pub height: u32,
    pub hash: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionStat {
    pub download_count: u64,
    pub rating_count: u64,
    pub rating: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReversingModelInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub nsfw: bool,
    pub poi: bool,
    pub mode: ModelMode,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersion {
    pub id: u64,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub model: Option<ReversingModelInfo>,
    #[serde(default)]
    pub model_id: Option<u64>,
    pub created_at: UtcDateTime,
    pub download_url: String,
    pub trained_words: Vec<String>,
    pub files: Vec<ModelVersionFile>,
    pub images: Vec<ModelVersionImage>,
    pub stats: ModelVersionStat,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelMeta {
    pub total_items: String,
    pub page_size: String,
    pub total_pages: String,
    pub next_page: String,
    pub prev_page: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: u64,
    pub description: String,
    pub nsfw: bool,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub tags: Vec<String>,
    pub mode: Option<ModelMode>,
    pub creator: Creator,
    pub stats: ModelStats,
    pub model_versions: Vec<ModelVersion>,
    pub metadata: ModelMeta,
}
