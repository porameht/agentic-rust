//! Presigned URL utilities for secure file access.

use crate::client::StorageClient;
use crate::error::StorageResult;
use crate::models::PresignedUrlOptions;
use serde::{Deserialize, Serialize};

/// Presigned URL response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    /// The presigned URL
    pub url: String,

    /// Expiration time in seconds
    pub expires_in: u32,

    /// The bucket name
    pub bucket: String,

    /// The object key
    pub key: String,

    /// HTTP method (GET for download, PUT for upload)
    pub method: String,
}

impl StorageClient {
    /// Generate a presigned URL for downloading a brochure
    pub async fn get_brochure_download_url(
        &self,
        bucket: &str,
        key: &str,
        filename: Option<&str>,
        expires_in: u32,
    ) -> StorageResult<PresignedUrl> {
        let options = if let Some(name) = filename {
            PresignedUrlOptions::new(expires_in).with_download_filename(name)
        } else {
            PresignedUrlOptions::new(expires_in)
        };

        let url = self.get_presigned_download_url(bucket, key, options).await?;

        Ok(PresignedUrl {
            url,
            expires_in,
            bucket: bucket.to_string(),
            key: key.to_string(),
            method: "GET".to_string(),
        })
    }

    /// Generate a presigned URL for uploading a file
    pub async fn get_upload_url(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        expires_in: u32,
    ) -> StorageResult<PresignedUrl> {
        let url = self
            .get_presigned_upload_url(bucket, key, expires_in, content_type)
            .await?;

        Ok(PresignedUrl {
            url,
            expires_in,
            bucket: bucket.to_string(),
            key: key.to_string(),
            method: "PUT".to_string(),
        })
    }
}

/// Builder for generating presigned URLs with various options
pub struct PresignedUrlBuilder<'a> {
    client: &'a StorageClient,
    bucket: String,
    key: String,
    expires_in: u32,
    content_disposition: Option<String>,
    content_type: Option<String>,
}

impl<'a> PresignedUrlBuilder<'a> {
    pub fn new(client: &'a StorageClient, bucket: impl Into<String>, key: impl Into<String>) -> Self {
        Self {
            client,
            bucket: bucket.into(),
            key: key.into(),
            expires_in: 3600,
            content_disposition: None,
            content_type: None,
        }
    }

    /// Set expiration time in seconds
    pub fn expires_in(mut self, seconds: u32) -> Self {
        self.expires_in = seconds;
        self
    }

    /// Set as downloadable with specific filename
    pub fn download_as(mut self, filename: impl Into<String>) -> Self {
        self.content_disposition = Some(format!("attachment; filename=\"{}\"", filename.into()));
        self
    }

    /// Set content type
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = Some(content_type.into());
        self
    }

    /// Build download URL
    pub async fn build_download(self) -> StorageResult<PresignedUrl> {
        let options = PresignedUrlOptions {
            expires_in: self.expires_in,
            content_disposition: self.content_disposition,
            content_type: self.content_type,
        };

        let url = self
            .client
            .get_presigned_download_url(&self.bucket, &self.key, options)
            .await?;

        Ok(PresignedUrl {
            url,
            expires_in: self.expires_in,
            bucket: self.bucket,
            key: self.key,
            method: "GET".to_string(),
        })
    }

    /// Build upload URL
    pub async fn build_upload(self) -> StorageResult<PresignedUrl> {
        let url = self
            .client
            .get_presigned_upload_url(
                &self.bucket,
                &self.key,
                self.expires_in,
                self.content_type.as_deref(),
            )
            .await?;

        Ok(PresignedUrl {
            url,
            expires_in: self.expires_in,
            bucket: self.bucket,
            key: self.key,
            method: "PUT".to_string(),
        })
    }
}
