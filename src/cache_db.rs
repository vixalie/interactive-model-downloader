use std::{
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex},
};

use anyhow::{Result, anyhow, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::{Value, to_vec};

use crate::civitai;

static CACHE_DB: LazyLock<Arc<Mutex<sled::Db>>> = LazyLock::new(|| {
    let cache_dir = directories::UserDirs::new()
        .map(|dirs| dirs.home_dir().to_path_buf())
        .map(|home_dir| home_dir.join(".config").join("imd").join("cache"));
    if cache_dir.is_none() {
        panic!("Failed to get cache directory.");
    }
    let cache_dir = cache_dir.unwrap();
    if !cache_dir.exists() {
        std::fs::create_dir_all(&cache_dir).expect("Failed to create cache directory");
    }

    let db_path = cache_dir.join("cache.db");
    let db = sled::open(&db_path).expect("Failed to open cache database");
    Arc::new(Mutex::new(db))
});

pub fn store_civitai_model(model_meta: &civitai::Model) -> Result<()> {
    let model_id = model_meta.id();
    let model_key = format!("civitai:model:{}", model_id);
    let db = CACHE_DB.lock()?;
    db.insert(model_key, model_meta.to_bytes())?;
    db.flush()?;
    Ok(())
}

pub fn retreive_civitai_model(model_id: u64) -> Result<Option<civitai::Model>> {
    let model_key = format!("civitai:model:{}", model_id);
    let db = CACHE_DB.lock()?;
    let raw_value = db.get(&model_key)?;
    match raw_value {
        Some(value) => {
            let model_meta_value: Value = serde_json::from_slice(&value)?;
            let model_meta = civitai::Model::try_from(&model_meta_value)?;
            Ok(Some(model_meta))
        }
        None => Ok(None),
    }
}

pub fn is_civitai_model_exists(model_id: u64) -> Result<bool> {
    let model_key = format!("civitai:model:{}", model_id);
    let db = CACHE_DB.lock()?;
    let exists = db.contains_key(&model_key)?;
    Ok(exists)
}

pub fn store_civitai_model_version(model_version_meta: &ModelVersion) -> Result<()> {
    let model_version_key = format!(
        "civitai:model:{}:{}",
        model_version_meta.model_id(),
        model_version_meta.id()
    );
    let db = CACHE_DB.lock()?;
    db.insert(&model_version_key, model_version_meta.to_bytes())?;
    db.flush()?;
    Ok(())
}

pub fn retreive_civitai_model_version(
    model_id: u64,
    model_version_id: u64,
) -> Result<Option<civitai::ModelVersion>> {
    let model_version_key = format!("civitai:model:{}:{}", model_id, model_version_id);
    let db = CACHE_DB.lock()?;
    let version_raw_value = db.get(&model_version_key)?;
    match version_raw_value {
        Some(value) => {
            let version_value: Value = serde_json::from_slice(&value)?;
            let model_version = civitai::ModelVersion::try_from(&version_meta_value)?;
            Ok(Some(model_version))
        }
        None => Ok(None),
    }
}

pub fn is_civitai_model_version_exists(model_id: u64, model_version_id: u64) -> Result<bool> {
    let model_version_key = format!("civitai:model:{}:{}", model_id, model_version_id);
    let db = CACHE_DB.lock()?;
    let exists = db.contains_key(&model_version_key)?;
    Ok(exists)
}

pub fn store_civitai_model_community_image(
    model_id: u64,
    image: &civitai::ModelCommunityImage,
) -> Result<()> {
    let model_community_images_key = format!("civitai:model:{}:image:{}", model_id, image.id());
    let db = CACHE_DB.lock()?;
    db.insert(model_community_images_key, image.to_bytes())?;
    db.flush()?;
    Ok(())
}

pub fn fetch_civitai_model_community_images(
    model_id: u64,
) -> Result<Vec<civitai::ModelCommunityImage>> {
    let model_community_images_prefix = format!("civitai:model:{}:image:", model_id);
    let db = CACHE_DB.lock()?;
    let mut images = db.scan_prefix(model_community_images_prefix);

    let mut community_images = Vec::new();
    for value in images {
        if let Ok((_, raw_value)) = value {
            let raw_image_value: Value = serde_json::from_slice(&raw_value)?;
            if let Ok(image) = civitai::ModelCommunityImage::try_from(raw_image_value) {
                community_images.push(image);
            }
        }
    }
    Ok(community_images)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CivitaiFileLocationRecord {
    pub model_id: u64,
    pub version_id: u64,
    pub locations: Vec<String>,
}

pub fn store_civitai_model_file_location<P: AsRef<Path>>(
    model_version: &civitai::ModelVersion,
    model_version_file: &civitai::ModelVersionFile,
    file_location: P,
) -> Result<()> {
    let location = file_location.as_ref().canonicalize()?;
    let location_str = location.to_string_lossy().into_owned();

    let file_id_key = format!(
        "civitai:model:file:{}:{}:id:{}",
        model_version.model_id(),
        model_version.id(),
        model_version_file.id()
    );
    let file_sha256_key = model_version_file
        .sha256_hash()
        .map(|hash| format!("civitai:model:file:sha256:{}", hash));
    let file_blake3_key = model_version_file
        .blake3_hash()
        .map(|hash| format!("civitai:model:file:blake3:{}", hash));

    let new_record = CivitaiFileLocationRecord {
        model_id: model_version.model_id(),
        version_id: model_version.id(),
        locations: vec![location_str],
    };

    let db = CACHE_DB.lock()?;
    if let Ok(Some(record)) = db.get(&file_id_key) {
        let mut record: CivitaiFileLocationRecord = serde_json::from_slice(&record)?;
        record.locations.push(location_str);
        db.insert(&file_id_key, serde_json::to_vec(&record)?)?;
    } else {
        db.insert(&file_id_key, serde_json::to_vec(&new_record)?)?;
    }
    if let Some(file_sha256_key) = file_sha256_key {
        if let Ok(Some(record)) = db.get(&file_sha256_key) {
            let mut record: CivitaiFileLocationRecord = serde_json::from_slice(&record)?;
            record.locations.push(location_str);
            db.insert(&file_sha256_key, serde_json::to_vec(&record)?)?;
        } else {
            db.insert(&file_sha256_key, serde_json::to_vec(&new_record)?)?;
        }
    }
    if let Some(file_blake3_key) = file_blake3_key {
        if let Ok(Some(record)) = db.get(&file_blake3_key) {
            let mut record: FileRecord = serde_json::from_slice(&record)?;
            record.locations.push(location_str);
            db.insert(&file_blake3_key, serde_json::to_vec(&record)?)?;
        } else {
            db.insert(&file_blake3_key, serde_json::to_vec(&new_record)?)?;
        }
    }
    db.flush()?;

    Ok(())
}

pub fn retreive_civitai_model_locations_by_ids(
    model_id: u64,
    version_id: u64,
    file_id: u64,
) -> Result<Vec<PathBuf>> {
    let location_key = format!(
        "civitai:model:file:{}:{}:id:{}",
        model_id, version_id, file_id
    );
    let db = CACHE_DB.lock()?;
    let record = db.get(&location_key)?;
    match record {
        Some(raw_value) => {
            let location_record: CivitaiFileLocationRecord = serde_json::from_slice(&raw_value)?;
            let converted_locations: Vec<PathBuf> = location_record
                .locations
                .iter()
                .map(|l| PathBuf::from(l))
                .collect();
            Ok(converted_locations)
        }
        None => Ok(None),
    }
}

pub fn retreive_civitai_model_locations_by_sha256(hash: String) -> Result<Option<Vec<PathBuf>>> {
    let location_key = format!("civitai:model:file:sha256:{}", hash);
    let db = CACHE_DB.lock()?;
    let record = db.get(&location_key)?;
    match record {
        Some(raw_value) => {
            let location_record: CivitaiFileLocationRecord = serde_json::from_slice(&raw_value)?;
            let converted_locations: Vec<PathBuf> = location_record
                .locations
                .iter()
                .map(|l| PathBuf::from(l))
                .collect();
            Ok(Some(converted_locations))
        }
        None => Ok(None),
    }
}

pub fn retreive_civitai_model_locations_by_blake3(hash: String) -> Result<Option<Vec<PathBuf>>> {
    let location_key = format!("civitai:model:file:blake3:{}", hash);
    let db = CACHE_DB.lock()?;
    let record = db.get(&location_key)?;
    match record {
        Some(raw_value) => {
            let location_record: CivitaiFileLocationRecord = serde_json::from_slice(&raw_value)?;
            let converted_locations: Vec<PathBuf> = location_record
                .locations
                .iter()
                .map(|l| PathBuf::from(l))
                .collect();
            Ok(Some(converted_locations))
        }
        None => Ok(None),
    }
}
