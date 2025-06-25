use thiserror::Error;

#[derive(Debug, Error)]
pub enum CivitaiParseError {
    #[error("Missing required field in {0}: {1}")]
    MissingRequiredField(String, String),
    #[error("Unregconized field in {0}: {1}")]
    UnregconizedField(String, String),
    #[error("Invalid field value in {0}: {1}")]
    InvalidFieldValue(String, String),
    #[error("Universal error: {0}")]
    AnyhowError(anyhow::Error),
}

impl From<anyhow::Error> for CivitaiParseError {
    fn from(err: anyhow::Error) -> Self {
        Self::AnyhowError(err)
    }
}
