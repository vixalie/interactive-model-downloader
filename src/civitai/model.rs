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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelVersionFileMeta {
    pub fp: Option<ModelFileFloatingPoints>,
    pub size: Option<ModelSize>,
    pub format: Option<ModelFileFormat>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelVersionFileHashes {
    #[serde(rename = "AutoV1")]
    pub auto_v1: Option<String>,
    #[serde(rename = "AutoV2")]
    pub auto_v2: Option<String>,
    #[serde(rename = "AutoV3")]
    pub auto_v3: Option<String>,
    #[serde(rename = "SHA256")]
    pub sha256: Option<String>,
    #[serde(rename = "CRC32")]
    pub crc32: Option<String>,
    #[serde(rename = "BLAKE3")]
    pub blake3: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionFile {
    pub id: u64,
    #[serde(rename = "sizeKB")]
    pub size_kb: f64,
    pub name: String,
    pub pickle_scan_result: FileScanResult,
    pub virus_scan_result: FileScanResult,
    #[serde(default)]
    pub scanned_at: Option<UtcDateTime>,
    pub primary: Option<bool>,
    pub metadata: ModelVersionFileMeta,
    pub hashes: ModelVersionFileHashes,
    pub download_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionImage {
    pub url: String,
    pub nsfw: String,
    pub width: u32,
    pub height: u32,
    pub hash: String,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelVersion {
    pub id: u64,
    pub index: u32,
    pub name: String,
    pub description: String,
    pub created_at: Option<UtcDateTime>,
    pub base_model: Option<String>,
    pub download_url: String,
    pub trained_words: Vec<String>,
    pub files: Vec<ModelVersionFile>,
    pub images: Vec<ModelVersionImage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub allow_no_credit: bool,
    pub nsfw: bool,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub tags: Vec<String>,
    pub mode: Option<ModelMode>,
    pub creator: Creator,
    pub model_versions: Vec<ModelVersion>,
}
