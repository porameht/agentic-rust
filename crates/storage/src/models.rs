use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    pub key: String,
    pub bucket: String,
    pub content_type: String,
    pub size: u64,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    pub key: String,
    pub bucket: String,
    pub etag: String,
    pub size: u64,
    pub content_type: String,
    pub url: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UploadOptions {
    pub content_type: Option<String>,
    pub key: Option<String>,
    pub public: bool,
    pub metadata: HashMap<String, String>,
    pub content_disposition: Option<String>,
}

impl UploadOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
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

    pub fn with_metadata(mut self, k: impl Into<String>, v: impl Into<String>) -> Self {
        self.metadata.insert(k.into(), v.into());
        self
    }

    pub fn with_download_filename(mut self, filename: impl Into<String>) -> Self {
        self.content_disposition = Some(format!("attachment; filename=\"{}\"", filename.into()));
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct PresignedUrlOptions {
    pub expires_in: u32,
    pub content_disposition: Option<String>,
    pub content_type: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjectsResult {
    pub objects: Vec<ObjectInfo>,
    pub prefixes: Vec<String>,
    pub is_truncated: bool,
    pub continuation_token: Option<String>,
}
