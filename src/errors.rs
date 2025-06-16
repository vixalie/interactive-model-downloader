use thiserror::Error;

#[derive(Debug, Error)]
pub enum CivitaiParseError {
    #[error("Unknown float point type")]
    UnknownFloatPoint,
    #[error("Unknown model size")]
    UnknownModelSize,
    #[error("Unknown model mode")]
    UnknownModelMode,
    #[error("Unknown model type")]
    UnknownModelType,
    #[error("Failed parsing model version file field: {0}")]
    FailedParsingModelVersionFileField(String),
    #[error("Failed parsing model version field: {0}")]
    FailedParsingModelVersionField(String),
    #[error("Failed retreiving model version field: {0}")]
    FailedRetreivingModelVersionField(String),
    #[error("Failed parsing model version file[{0}], field {1}")]
    FailedParsingModelVersionFile(usize, String),
    #[error("Failed parsing model field: {0}")]
    FailedParsingModelField(String),
    #[error("Failed retreiving model field: {0}")]
    FailedRetreivingModelField(String),
    #[error("Failed parsing version field {0} in model: {1}")]
    FailedParsingVersionFieldInModel(usize, String),
    #[error("Universal error: {0}")]
    AnyhowError(anyhow::Error),
}

impl From<anyhow::Error> for CivitaiParseError {
    fn from(err: anyhow::Error) -> Self {
        Self::AnyhowError(err)
    }
}
