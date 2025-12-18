use crate::client::StorageClient;
use crate::error::StorageResult;
use crate::models::PresignedUrlOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresignedUrl {
    pub url: String,
    pub expires_in: u32,
    pub bucket: String,
    pub key: String,
    pub method: String,
}

impl StorageClient {
    pub async fn download_url(
        &self,
        bucket: &str,
        key: &str,
        filename: Option<&str>,
        expires_in: u32,
    ) -> StorageResult<PresignedUrl> {
        let opts = match filename {
            Some(name) => PresignedUrlOptions::new(expires_in).with_download_filename(name),
            None => PresignedUrlOptions::new(expires_in),
        };

        Ok(PresignedUrl {
            url: self.presigned_download(bucket, key, opts).await?,
            expires_in,
            bucket: bucket.into(),
            key: key.into(),
            method: "GET".into(),
        })
    }

    pub async fn upload_url(
        &self,
        bucket: &str,
        key: &str,
        content_type: Option<&str>,
        expires_in: u32,
    ) -> StorageResult<PresignedUrl> {
        Ok(PresignedUrl {
            url: self.presigned_upload(bucket, key, expires_in, content_type).await?,
            expires_in,
            bucket: bucket.into(),
            key: key.into(),
            method: "PUT".into(),
        })
    }
}

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

    pub fn expires_in(mut self, seconds: u32) -> Self {
        self.expires_in = seconds;
        self
    }

    pub fn download_as(mut self, filename: impl Into<String>) -> Self {
        self.content_disposition = Some(format!("attachment; filename=\"{}\"", filename.into()));
        self
    }

    pub fn content_type(mut self, ct: impl Into<String>) -> Self {
        self.content_type = Some(ct.into());
        self
    }

    pub async fn build_download(self) -> StorageResult<PresignedUrl> {
        let opts = PresignedUrlOptions {
            expires_in: self.expires_in,
            content_disposition: self.content_disposition,
            content_type: self.content_type,
        };

        Ok(PresignedUrl {
            url: self.client.presigned_download(&self.bucket, &self.key, opts).await?,
            expires_in: self.expires_in,
            bucket: self.bucket,
            key: self.key,
            method: "GET".into(),
        })
    }

    pub async fn build_upload(self) -> StorageResult<PresignedUrl> {
        Ok(PresignedUrl {
            url: self
                .client
                .presigned_upload(&self.bucket, &self.key, self.expires_in, self.content_type.as_deref())
                .await?,
            expires_in: self.expires_in,
            bucket: self.bucket,
            key: self.key,
            method: "PUT".into(),
        })
    }
}
