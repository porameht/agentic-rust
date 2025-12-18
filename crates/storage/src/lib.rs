//! S3-compatible Object Storage Client
//!
//! This crate provides a unified interface for storing and retrieving objects
//! from S3-compatible storage systems including:
//! - RustFS (recommended - high performance, Apache 2.0 license)
//! - MinIO
//! - AWS S3
//! - Any S3-compatible storage
//!
//! # Example
//!
//! ```rust,ignore
//! use storage::{StorageClient, StorageConfig};
//!
//! let config = StorageConfig::new("http://localhost:9000", "access_key", "secret_key");
//! let client = StorageClient::new(config).await?;
//!
//! // Upload a file
//! let object = client.upload_file("brochures", "/path/to/file.pdf").await?;
//!
//! // Get presigned download URL
//! let url = client.get_presigned_url("brochures", &object.key, 3600).await?;
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod models;
pub mod presigned;

pub use client::StorageClient;
pub use config::StorageConfig;
pub use error::{StorageError, StorageResult};
pub use models::{ObjectInfo, UploadResult};
