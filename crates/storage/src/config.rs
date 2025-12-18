//! Storage configuration.

use serde::{Deserialize, Serialize};

/// Storage configuration for S3-compatible services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Storage endpoint URL (e.g., "http://localhost:9000" for RustFS/MinIO)
    pub endpoint: String,

    /// Access key ID
    pub access_key: String,

    /// Secret access key
    pub secret_key: String,

    /// AWS region (use "us-east-1" for most S3-compatible services)
    #[serde(default = "default_region")]
    pub region: String,

    /// Use path-style URLs (required for most S3-compatible services)
    #[serde(default = "default_path_style")]
    pub path_style: bool,

    /// Default bucket for uploads
    #[serde(default)]
    pub default_bucket: Option<String>,

    /// Base URL for public access (for generating public URLs)
    #[serde(default)]
    pub public_url: Option<String>,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

fn default_path_style() -> bool {
    true
}

impl StorageConfig {
    /// Create a new storage configuration
    pub fn new(
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region: default_region(),
            path_style: default_path_style(),
            default_bucket: None,
            public_url: None,
        }
    }

    /// Create configuration for RustFS
    pub fn rustfs(endpoint: impl Into<String>, access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self::new(endpoint, access_key, secret_key)
            .with_path_style(true)
    }

    /// Create configuration for MinIO
    pub fn minio(endpoint: impl Into<String>, access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self::new(endpoint, access_key, secret_key)
            .with_path_style(true)
    }

    /// Create configuration for AWS S3
    pub fn aws_s3(region: impl Into<String>, access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        let region = region.into();
        Self {
            endpoint: format!("https://s3.{}.amazonaws.com", region),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            region,
            path_style: false,
            default_bucket: None,
            public_url: None,
        }
    }

    /// Set the region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Set path style
    pub fn with_path_style(mut self, path_style: bool) -> Self {
        self.path_style = path_style;
        self
    }

    /// Set default bucket
    pub fn with_default_bucket(mut self, bucket: impl Into<String>) -> Self {
        self.default_bucket = Some(bucket.into());
        self
    }

    /// Set public URL base
    pub fn with_public_url(mut self, url: impl Into<String>) -> Self {
        self.public_url = Some(url.into());
        self
    }

    /// Load configuration from environment variables
    pub fn from_env() -> crate::StorageResult<Self> {
        Ok(Self {
            endpoint: std::env::var("STORAGE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            access_key: std::env::var("STORAGE_ACCESS_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            secret_key: std::env::var("STORAGE_SECRET_KEY")
                .unwrap_or_else(|_| "minioadmin".to_string()),
            region: std::env::var("STORAGE_REGION")
                .unwrap_or_else(|_| default_region()),
            path_style: std::env::var("STORAGE_PATH_STYLE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            default_bucket: std::env::var("STORAGE_DEFAULT_BUCKET").ok(),
            public_url: std::env::var("STORAGE_PUBLIC_URL").ok(),
        })
    }
}
