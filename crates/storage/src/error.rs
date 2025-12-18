//! Storage error types.

use thiserror::Error;

/// Result type alias for storage operations
pub type StorageResult<T> = Result<T, StorageError>;

/// Storage error types
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Bucket error: {0}")]
    Bucket(String),

    #[error("Object not found: {bucket}/{key}")]
    NotFound { bucket: String, key: String },

    #[error("Upload failed: {0}")]
    Upload(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Delete failed: {0}")]
    Delete(String),

    #[error("Presigned URL generation failed: {0}")]
    PresignedUrl(String),

    #[error("Invalid file: {0}")]
    InvalidFile(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("S3 error: {0}")]
    S3(String),
}

impl From<s3::error::S3Error> for StorageError {
    fn from(err: s3::error::S3Error) -> Self {
        StorageError::S3(err.to_string())
    }
}
