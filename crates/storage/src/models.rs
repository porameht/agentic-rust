//! Storage data models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a stored object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    /// Object key (path in bucket)
    pub key: String,

    /// Bucket name
    pub bucket: String,

    /// Content type (MIME type)
    pub content_type: String,

    /// Size in bytes
    pub size: u64,

    /// ETag (usually MD5 hash)
    pub etag: Option<String>,

    /// Last modified timestamp
    pub last_modified: Option<DateTime<Utc>>,

    /// Custom metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Result of an upload operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// Object key
    pub key: String,

    /// Bucket name
    pub bucket: String,

    /// ETag
    pub etag: String,

    /// Size in bytes
    pub size: u64,

    /// Content type
    pub content_type: String,

    /// Public URL (if available)
    pub url: Option<String>,

    /// SHA256 hash of content
    pub sha256: Option<String>,
}

/// Options for upload operations
#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    /// Custom content type (auto-detected if not specified)
    pub content_type: Option<String>,

    /// Custom key (auto-generated if not specified)
    pub key: Option<String>,

    /// Whether to make the object publicly readable
    pub public: bool,

    /// Custom metadata
    pub metadata: std::collections::HashMap<String, String>,

    /// Content disposition (for download filename)
    pub content_disposition: Option<String>,
}

impl UploadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    pub fn public(mut self) -> Self {
        self.public = true;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    pub fn with_download_filename(mut self, filename: impl Into<String>) -> Self {
        self.content_disposition = Some(format!("attachment; filename=\"{}\"", filename.into()));
        self
    }
}

/// Options for presigned URL generation
#[derive(Debug, Clone)]
pub struct PresignedUrlOptions {
    /// Expiration time in seconds
    pub expires_in: u32,

    /// Content disposition override
    pub content_disposition: Option<String>,

    /// Content type override
    pub content_type: Option<String>,
}

impl Default for PresignedUrlOptions {
    fn default() -> Self {
        Self {
            expires_in: 3600, // 1 hour
            content_disposition: None,
            content_type: None,
        }
    }
}

impl PresignedUrlOptions {
    pub fn new(expires_in: u32) -> Self {
        Self {
            expires_in,
            ..Default::default()
        }
    }

    pub fn with_download_filename(mut self, filename: impl Into<String>) -> Self {
        self.content_disposition = Some(format!("attachment; filename=\"{}\"", filename.into()));
        self
    }
}

/// List objects result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsResult {
    /// Objects in the bucket
    pub objects: Vec<ObjectInfo>,

    /// Common prefixes (for hierarchical listing)
    pub prefixes: Vec<String>,

    /// Whether there are more results
    pub is_truncated: bool,

    /// Continuation token for pagination
    pub continuation_token: Option<String>,
}
