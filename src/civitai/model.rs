use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::errors::CivitaiParseError;

macro_rules! match_enum_variants {
    ($enum_type:ident { $($str_val:expr => $variant:ident),* }, $error_variant:path) => {
        impl TryFrom<&Value> for $enum_type {
            type Error = CivitaiParseError;

            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value.as_str() {
                    $(
                        Some(s) if s.eq_ignore_ascii_case($str_val) => Ok($enum_type::$variant),
                    )*
                    _ => Err($error_variant)
                }
            }
        }
    };
}

macro_rules! get_field {
    ($value:expr, $field:expr, $error_variant:path) => {{
        $value
            .get($field)
            .ok_or_else(|| $error_variant($field.into()))
    }};
    ($value:expr, $field:expr, $parser:expr) => {{ $value.get($field).and_then($parser) }};
    ($value:expr, $field:expr, $parser:expr, $error_variant:path) => {{
        $value
            .get($field)
            .and_then($parser)
            .ok_or_else(|| $error_variant($field.into()))
    }};
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    /// Main Model
    Checkpoint,
    /// aka: Embedding Model
    TextualInversion,
    Hypernetwork,
    AestheticGradient,
    /// LoRA Model
    LoRA,
    /// LyCORIS Model
    LyCORIS,
    DoRA,
    Controlnet,
    Poses,
    VAE,
    Upscaler,
    Detection,
    Wildcards,
    Workflows,
    Other,
}

match_enum_variants! {
    ModelType {
        "checkpoint" => Checkpoint,
        "textual_inversion" => TextualInversion,
        "textualinversion" => TextualInversion,
        "hypernetwork" => Hypernetwork,
        "aesthetic_gradient" => AestheticGradient,
        "lora" => LoRA,
        "locon" => LyCORIS,
        "dora" => DoRA,
        "controlnet" => Controlnet,
        "poses" => Poses,
        "vae" => VAE,
        "upscaler" => Upscaler,
        "detection" => Detection,
        "wildcards" => Wildcards,
        "workflows" => Workflows,
        "other" => Other
    },
    CivitaiParseError::UnknownModelType
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelMode {
    Archived,
    TakeDown,
}

match_enum_variants! {
    ModelMode {
        "archived" => Archived,
        "takedown" => TakeDown
    },
    CivitaiParseError::UnknownModelMode
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub username: String,
    pub image: Option<String>,
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

impl TryFrom<&Value> for ModelVersionFileHashes {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let hashes = serde_json::from_value::<ModelVersionFileHashes>(value.clone())?;
        Ok(hashes)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVersionFile {
    pub id: u64,
    #[serde(rename = "sizeKB")]
    pub size_kb: f64,
    pub name: String,
    pub primary: Option<bool>,
    pub hashes: ModelVersionFileHashes,
    pub download_url: String,
}

impl TryFrom<&Value> for ModelVersionFile {
    type Error = CivitaiParseError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let id = get_field!(
            value,
            "id",
            |v: &Value| v.as_u64(),
            CivitaiParseError::FailedParsingModelVersionFileField
        )?;
        let file_size = get_field!(
            value,
            "sizeKB",
            |v: &Value| v.as_f64(),
            CivitaiParseError::FailedParsingModelVersionFileField
        )?;
        let name = get_field!(
            value,
            "name",
            |v: &Value| v.as_str().map(ToString::to_string),
            CivitaiParseError::FailedParsingModelVersionFileField
        )?;
        let primary = get_field!(value, "primary", |v: &Value| v.as_bool());
        let hashes = value
            .get("hashes")
            .map(ModelVersionFileHashes::try_from)
            .transpose()?
            .ok_or(CivitaiParseError::FailedParsingModelVersionFileField(
                "hashes".into(),
            ))?;
        let download_url = get_field!(
            value,
            "downloadUrl",
            |v: &Value| v.as_str().map(ToString::to_string),
            CivitaiParseError::FailedParsingModelVersionFileField
        )?;
        Ok(ModelVersionFile {
            id,
            size_kb: file_size,
            name,
            primary,
            hashes,
            download_url,
        })
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ModelVersion {
    pub id: u64,
    pub index: u32,
    pub name: String,
    pub description: Option<String>,
    pub base_model: Option<String>,
    pub download_url: String,
    pub trained_words: Vec<String>,
    pub files: Vec<ModelVersionFile>,
}

impl TryFrom<&Value> for ModelVersion {
    type Error = CivitaiParseError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let id = get_field!(
            value,
            "id",
            |v: &Value| v.as_u64(),
            CivitaiParseError::FailedParsingModelVersionField
        )?;
        let index = get_field!(
            value,
            "index",
            |v: &Value| v.as_u64().map(|i| i as u32),
            CivitaiParseError::FailedParsingModelVersionField
        )?;
        let name = get_field!(
            value,
            "name",
            |v: &Value| v.as_str().map(ToString::to_string),
            CivitaiParseError::FailedParsingModelVersionField
        )?;
        let description = value
            .get("description")
            .and_then(|v| v.as_str().map(ToString::to_string));
        let base_model = get_field!(value, "baseModel", |v: &Value| v
            .as_str()
            .map(ToString::to_string));
        let download_url = get_field!(
            value,
            "downloadUrl",
            |v: &Value| v.as_str().map(ToString::to_string),
            CivitaiParseError::FailedParsingModelVersionField
        )?;
        let raw_words = value.get("trainedWords");
        let trained_words = if let Some(Value::Array(words)) = raw_words {
            let mut ret_words = Vec::new();
            for word in words {
                if let Value::String(word) = word {
                    ret_words.push(word.to_string());
                }
            }
            ret_words
        } else {
            Vec::new()
        };
        let raw_files =
            value
                .get("files")
                .ok_or(CivitaiParseError::FailedRetreivingModelVersionField(
                    "files".into(),
                ))?;
        let version_files = if let Value::Array(files) = raw_files {
            let mut ret_files = Vec::new();
            for (index, file_info) in files.iter().enumerate() {
                let parsed_info = ModelVersionFile::try_from(file_info).map_err(|e| {
                    if let CivitaiParseError::FailedParsingModelVersionFileField(field_name) = e {
                        CivitaiParseError::FailedParsingModelVersionFile(index, field_name)
                    } else {
                        CivitaiParseError::FailedRetreivingModelVersionField("files".into())
                    }
                })?;
                ret_files.push(parsed_info);
            }
            ret_files
        } else {
            return Err(CivitaiParseError::FailedParsingModelVersionField(
                "files".into(),
            ));
        };
        Ok(ModelVersion {
            id,
            index,
            name,
            description,
            base_model,
            download_url,
            trained_words,
            files: version_files,
        })
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub nsfw: bool,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub tags: Vec<String>,
    pub mode: Option<ModelMode>,
    pub creator: Option<Creator>,
    pub model_versions: Vec<ModelVersion>,
}

impl TryFrom<&Value> for Model {
    type Error = CivitaiParseError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let id = get_field!(
            value,
            "id",
            |v: &Value| v.as_u64(),
            CivitaiParseError::FailedParsingModelField
        )?;
        let name = get_field!(
            value,
            "name",
            |v: &Value| v.as_str().map(ToString::to_string),
            CivitaiParseError::FailedParsingModelField
        )?;
        let description = get_field!(
            value,
            "description",
            |v: &Value| v.as_str().map(ToString::to_string),
            CivitaiParseError::FailedParsingModelField
        )?;
        let nsfw = get_field!(
            value,
            "nsfw",
            |v: &Value| v.as_bool(),
            CivitaiParseError::FailedParsingModelField
        )?;
        let model_type = value
            .get("type")
            .map(ModelType::try_from)
            .transpose()?
            .ok_or_else(|| CivitaiParseError::FailedParsingModelField("type".into()))?;
        let mode = value
            .get("mode")
            .map(ModelMode::try_from)
            .transpose()
            .map_err(|_| CivitaiParseError::FailedParsingModelField("mode".into()))?;
        let creator = value
            .get("creator")
            .map(|v: &Value| serde_json::from_value::<Creator>(v.clone()))
            .transpose()
            .map_err(|_| CivitaiParseError::FailedParsingModelField("creator".into()))?;
        let raw_tags = get_field!(value, "tags", CivitaiParseError::FailedRetreivingModelField)?;
        let tags = if let Value::Array(tags) = raw_tags {
            let mut ret_tags = Vec::new();
            for tag in tags {
                if let Value::String(tag) = tag {
                    ret_tags.push(tag.to_string());
                }
            }
            ret_tags
        } else {
            return Err(CivitaiParseError::FailedParsingModelField("tags".into()));
        };
        let raw_versions = get_field!(
            value,
            "modelVersions",
            CivitaiParseError::FailedRetreivingModelField
        )?;
        let versions = if let Value::Array(versions) = raw_versions {
            let mut ret_versions = Vec::new();
            for (index, version) in versions.iter().enumerate() {
                match ModelVersion::try_from(version) {
                    Ok(version) => {
                        ret_versions.push(version);
                    }
                    Err(e) => {
                        return Err(CivitaiParseError::FailedParsingVersionFieldInModel(
                            index,
                            e.to_string(),
                        ));
                    }
                }
            }
            ret_versions
        } else {
            return Err(CivitaiParseError::FailedParsingModelField(
                "versions".into(),
            ));
        };

        Ok(Model {
            id,
            name,
            description,
            nsfw,
            model_type,
            tags,
            mode,
            creator,
            model_versions: versions,
        })
    }
}
