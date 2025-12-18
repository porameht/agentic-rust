use thiserror::Error;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("config: {0}")]
    Config(String),
    #[error("connection: {0}")]
    Connection(String),
    #[error("bucket: {0}")]
    Bucket(String),
    #[error("not found: {bucket}/{key}")]
    NotFound { bucket: String, key: String },
    #[error("upload: {0}")]
    Upload(String),
    #[error("download: {0}")]
    Download(String),
    #[error("delete: {0}")]
    Delete(String),
    #[error("presigned url: {0}")]
    PresignedUrl(String),
    #[error("invalid file: {0}")]
    InvalidFile(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("s3: {0}")]
    S3(String),
}

impl From<s3::error::S3Error> for StorageError {
    fn from(err: s3::error::S3Error) -> Self {
        StorageError::S3(err.to_string())
    }
}
