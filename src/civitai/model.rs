use serde_json::Value;
use time::{UtcDateTime, format_description::well_known::Rfc3339};

use crate::errors::CivitaiParseError;

pub struct Model(Value);
pub struct ModelVersionBrief(Value);
pub struct ModelVersion(Value);
pub struct ModelVersionFile(Value);
pub struct ModelImage(Value);
pub struct ModelCommunityImage(Value);

pub trait ImageMeta {
    fn sampler(&self) -> Option<String>;
    fn scheduler(&self) -> Option<String>;
    fn seed(&self) -> Option<u64>;
    fn steps(&self) -> Option<u64>;
    fn cfg_scale(&self) -> Option<f64>;
    fn denoising_strength(&self) -> Option<f64>;
    fn use_model(&self) -> Option<String>;
    fn use_model_version(&self) -> Option<String>;
    fn positive_prompt(&self) -> Option<String>;
    fn negative_prompt(&self) -> Option<String>;
}

macro_rules! ensure_required_field {
    ($value:expr, $struct_name:expr, $field_name:expr) => {
        if $value[$field_name].is_null() {
            return Err(crate::errors::CivitaiParseError::MissingRequiredField(
                $struct_name.to_string(),
                $field_name.to_string(),
            ));
        }
    };
}

macro_rules! impl_try_from_value_for_meta {
    ($struct_name:ident, $($field_name:expr),+) => {
        impl TryFrom<&Value> for $struct_name {
            type Error = CivitaiParseError;

            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                $(
                    ensure_required_field!(value, stringify!($struct_name), $field_name);
                )+
                Ok(Self(value.clone()))
            }
        }
    };
}

impl_try_from_value_for_meta!(Model, "id", "name", "description", "modelVersions");
impl_try_from_value_for_meta!(ModelVersionBrief, "id", "name", "index");
impl_try_from_value_for_meta!(ModelVersion, "id", "modelId", "name", "files", "images");
impl_try_from_value_for_meta!(ModelVersionFile, "id", "sizeKB", "name", "downloadUrl");
impl_try_from_value_for_meta!(ModelImage, "url", "hasMeta", "hasPositivePrompt");
impl_try_from_value_for_meta!(ModelCommunityImage, "id", "url");

impl Model {
    pub fn id(&self) -> u64 {
        self.0["id"].as_u64().unwrap()
    }

    pub fn name(&self) -> String {
        self.0["name"].as_str().map(String::from).unwrap()
    }

    pub fn description(&self) -> String {
        self.0["description"].as_str().map(String::from).unwrap()
    }

    pub fn markdown_description(&self) -> String {
        self.0["description"]
            .as_str()
            .map(html2md::parse_html)
            .unwrap()
    }

    pub fn versions(&self) -> Result<Vec<ModelVersionBrief>, CivitaiParseError> {
        let versions = &self.0["modelVersions"];
        if !versions.is_array() {
            return Err(CivitaiParseError::UnregconizedField(
                "Model".to_string(),
                "modelVersions".to_string(),
            ));
        }

        let mut collected_versions = Vec::new();
        for version in versions.as_array().unwrap().iter() {
            let parsed_version = ModelVersionBrief::try_from(version)?;
            collected_versions.push(parsed_version);
        }

        Ok(collected_versions)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap()
    }
}

impl ModelVersionBrief {
    pub fn id(&self) -> u64 {
        self.0["id"].as_u64().unwrap()
    }

    pub fn index(&self) -> u64 {
        self.0["index"].as_u64().unwrap()
    }

    pub fn name(&self) -> String {
        self.0["name"].as_str().map(String::from).unwrap()
    }

    pub fn description(&self) -> Option<String> {
        self.0["description"].as_str().map(String::from)
    }

    pub fn choice(&self) -> (u64, String) {
        (self.id(), self.name())
    }
}

impl ModelVersion {
    pub fn id(&self) -> u64 {
        self.0["id"].as_u64().unwrap()
    }

    pub fn model_id(&self) -> u64 {
        self.0["modelId"].as_u64().unwrap()
    }

    pub fn name(&self) -> String {
        self.0["name"].as_str().map(String::from).unwrap()
    }

    pub fn model_name(&self) -> Option<String> {
        self.0["model"]["name"].as_str().map(String::from)
    }

    pub fn description(&self) -> Option<String> {
        self.0["description"].as_str().map(String::from)
    }

    pub fn markdown_description(&self) -> Option<String> {
        self.0["description"].as_str().map(html2md::parse_html)
    }

    pub fn air(&self) -> Option<String> {
        self.0["air"].as_str().map(String::from)
    }

    pub fn is_early_access(&self) -> bool {
        let early_access_ends_str = &self.0["earlyAccessEndsAt"];
        if early_access_ends_str.is_null() {
            return false;
        }
        let early_access_ends_at = early_access_ends_str
            .as_str()
            .and_then(|s| UtcDateTime::parse(s, &Rfc3339).ok());
        if let Some(ends_at) = early_access_ends_at {
            return UtcDateTime::now() <= ends_at;
        } else {
            return false;
        }
    }

    pub fn trained_words(&self) -> Vec<String> {
        let mut trained_words = Vec::new();
        let words = &self.0["trainedWords"];
        if !words.is_array() {
            return trained_words;
        }

        for word in words.as_array().unwrap() {
            if let Some(w) = word.as_str().map(String::from) {
                trained_words.push(w);
            }
        }

        trained_words
    }

    pub fn files(&self) -> Result<Vec<ModelVersionFile>, CivitaiParseError> {
        let files = &self.0["files"];
        if !files.is_array() {
            return Err(CivitaiParseError::InvalidFieldValue(
                "ModelVersion".to_string(),
                "files".to_string(),
            ));
        }

        let mut version_files = Vec::new();
        for file in files.as_array().unwrap().iter() {
            let f = ModelVersionFile::try_from(file)?;
            version_files.push(f);
        }
        Ok(version_files)
    }

    pub fn images(&self) -> Result<Vec<ModelImage>, CivitaiParseError> {
        let images = &self.0["images"];
        if !images.is_array() {
            return Err(CivitaiParseError::InvalidFieldValue(
                "ModelVersion".to_string(),
                "images".to_string(),
            ));
        }

        let mut version_images = Vec::new();
        for image in images.as_array().unwrap() {
            let i = ModelImage::try_from(image)?;
            version_images.push(i);
        }
        Ok(version_images)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap()
    }
}

impl ModelVersionFile {
    pub fn id(&self) -> u64 {
        self.0["id"].as_u64().unwrap()
    }

    pub fn size(&self) -> f64 {
        self.0["sizeKB"].as_f64().unwrap()
    }

    pub fn name(&self) -> String {
        self.0["name"].as_str().map(String::from).unwrap()
    }

    pub fn download_url(&self) -> String {
        self.0["downloadUrl"].as_str().map(String::from).unwrap()
    }

    pub fn is_primary(&self) -> Option<bool> {
        self.0["primary"].as_bool()
    }

    pub fn blake3_hash(&self) -> Option<String> {
        self.0["hashes"]["BLAKE3"].as_str().map(String::from)
    }

    pub fn sha256_hash(&self) -> Option<String> {
        self.0["hashes"]["SHA256"]
            .as_str()
            .map(String::from)
            .map(|s| s.to_lowercase())
    }

    pub fn crc32(&self) -> Option<String> {
        self.0["hashes"]["CRC32"]
            .as_str()
            .map(String::from)
            .map(|s| s.to_lowercase())
    }

    pub fn choice(&self) -> (u64, String) {
        (self.id(), self.name())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap()
    }

    pub fn match_by_blake3(&self, blake3_str: &str) -> bool {
        self.blake3_hash()
            .map(|hash| hash.eq_ignore_ascii_case(blake3_str))
            .unwrap_or_default()
    }
}

impl ModelImage {
    pub fn url(&self) -> String {
        self.0["url"].as_str().map(String::from).unwrap()
    }

    pub fn media_type(&self) -> String {
        self.0["type"].as_str().map(String::from).unwrap()
    }

    pub fn hash(&self) -> Option<String> {
        self.0["hash"].as_str().map(String::from)
    }

    pub fn has_meta(&self) -> bool {
        self.0["hasMeta"].as_bool().unwrap_or_default()
    }

    pub fn has_positive_prompt(&self) -> bool {
        self.0["hasPositivePrompt"].as_bool().unwrap_or_default()
    }
}

impl ImageMeta for ModelImage {
    fn sampler(&self) -> Option<String> {
        self.0["meta"]["sampler"].as_str().map(String::from)
    }

    fn scheduler(&self) -> Option<String> {
        self.0["meta"]["scheduler"].as_str().map(String::from)
    }

    fn seed(&self) -> Option<u64> {
        self.0["meta"]["seed"].as_u64()
    }

    fn steps(&self) -> Option<u64> {
        self.0["meta"]["steps"].as_u64()
    }

    fn cfg_scale(&self) -> Option<f64> {
        self.0["meta"]["cfgScale"].as_f64()
    }

    fn denoising_strength(&self) -> Option<f64> {
        self.0["meta"]["Denoising strength"].as_f64()
    }

    fn use_model(&self) -> Option<String> {
        self.0["meta"]["Model"].as_str().map(String::from)
    }

    fn use_model_version(&self) -> Option<String> {
        self.0["meta"]["Version"].as_str().map(String::from)
    }

    fn positive_prompt(&self) -> Option<String> {
        self.0["meta"]["prompt"].as_str().map(String::from)
    }

    fn negative_prompt(&self) -> Option<String> {
        self.0["meta"]["negativePrompt"].as_str().map(String::from)
    }
}

pub fn try_parse_community_images(
    value: &Value,
) -> Result<Vec<ModelCommunityImage>, CivitaiParseError> {
    let items = &value["items"];
    if !items.is_array() {
        return Err(CivitaiParseError::InvalidFieldValue(
            "CommunityImages".to_string(),
            "items".to_string(),
        ));
    }

    let mut images = Vec::new();
    for item in items.as_array().unwrap() {
        let i = ModelCommunityImage::try_from(item)?;
        images.push(i);
    }
    Ok(images)
}

impl ModelCommunityImage {
    pub fn id(&self) -> u64 {
        self.0["id"].as_u64().unwrap()
    }

    pub fn url(&self) -> String {
        self.0["url"].as_str().map(String::from).unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self.0).unwrap()
    }
}

impl ImageMeta for ModelCommunityImage {
    fn seed(&self) -> Option<u64> {
        self.0["meta"]["seed"].as_u64()
    }

    fn sampler(&self) -> Option<String> {
        self.0["meta"]["sampler"].as_str().map(String::from)
    }

    fn scheduler(&self) -> Option<String> {
        self.0["meta"]["Schedule type"].as_str().map(String::from)
    }

    fn steps(&self) -> Option<u64> {
        self.0["meta"]["steps"].as_u64()
    }

    fn cfg_scale(&self) -> Option<f64> {
        self.0["meta"]["cfgScale"].as_f64()
    }

    fn denoising_strength(&self) -> Option<f64> {
        self.0["meta"]["Denoising strength"].as_f64()
    }

    fn use_model(&self) -> Option<String> {
        self.0["meta"]["Model"].as_str().map(String::from)
    }

    fn use_model_version(&self) -> Option<String> {
        self.0["meta"]["Version"].as_str().map(String::from)
    }

    fn positive_prompt(&self) -> Option<String> {
        self.0["meta"]["prompt"].as_str().map(String::from)
    }

    fn negative_prompt(&self) -> Option<String> {
        self.0["meta"]["negativePrompt"].as_str().map(String::from)
    }
}
