//! S3-compatible Storage Client for RustFS, MinIO, AWS S3.
//!
//! RustFS is recommended for high-performance, Apache 2.0 licensed storage.
//! See: https://github.com/rustfs/rustfs

use crate::config::StorageConfig;
use crate::error::{StorageError, StorageResult};
use crate::models::{ListObjectsResult, ObjectInfo, PresignedUrlOptions, UploadOptions, UploadResult};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

/// Storage client for S3-compatible object storage
///
/// Supports:
/// - RustFS (recommended) - https://rustfs.com
/// - MinIO
/// - AWS S3
/// - Any S3-compatible storage
#[derive(Clone)]
pub struct StorageClient {
    config: StorageConfig,
}

impl StorageClient {
    /// Create a new storage client
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// Create a storage client from environment variables
    pub fn from_env() -> StorageResult<Self> {
        let config = StorageConfig::from_env()?;
        Ok(Self::new(config))
    }

    /// Create a bucket handle
    fn create_bucket(&self, bucket_name: &str) -> StorageResult<Box<Bucket>> {
        let region = Region::Custom {
            region: self.config.region.clone(),
            endpoint: self.config.endpoint.clone(),
        };

        let credentials = Credentials::new(
            Some(&self.config.access_key),
            Some(&self.config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| StorageError::Config(e.to_string()))?;

        let mut bucket = Bucket::new(bucket_name, region, credentials)
            .map_err(|e| StorageError::Bucket(e.to_string()))?;

        if self.config.path_style {
            bucket = bucket.with_path_style();
        }

        Ok(bucket)
    }

    /// Check if a bucket exists
    pub async fn bucket_exists(&self, bucket_name: &str) -> StorageResult<bool> {
        let bucket = self.create_bucket(bucket_name)?;
        match bucket.head_object("/").await {
            Ok(_) => Ok(true),
            Err(s3::error::S3Error::HttpFailWithBody(404, _)) => Ok(false),
            Err(s3::error::S3Error::HttpFailWithBody(403, _)) => Ok(true), // Exists but no permission
            Err(e) => {
                // Try listing to check existence
                match bucket.list("".to_string(), Some("/".to_string())).await {
                    Ok(_) => Ok(true),
                    Err(_) => Err(StorageError::Bucket(e.to_string())),
                }
            }
        }
    }

    /// Create a bucket if it doesn't exist
    pub async fn create_bucket_if_not_exists(&self, bucket_name: &str) -> StorageResult<()> {
        let region = Region::Custom {
            region: self.config.region.clone(),
            endpoint: self.config.endpoint.clone(),
        };

        let credentials = Credentials::new(
            Some(&self.config.access_key),
            Some(&self.config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| StorageError::Config(e.to_string()))?;

        match Bucket::create_with_path_style(
            bucket_name,
            region,
            credentials,
            s3::BucketConfiguration::default(),
        )
        .await
        {
            Ok(_) => {
                info!(bucket = bucket_name, "Bucket created");
                Ok(())
            }
            Err(s3::error::S3Error::HttpFailWithBody(409, _)) => {
                debug!(bucket = bucket_name, "Bucket already exists");
                Ok(())
            }
            Err(e) => Err(StorageError::Bucket(e.to_string())),
        }
    }

    /// Upload bytes to storage
    pub async fn upload_bytes(
        &self,
        bucket_name: &str,
        data: &[u8],
        options: UploadOptions,
    ) -> StorageResult<UploadResult> {
        let bucket = self.create_bucket(bucket_name)?;

        // Generate key if not provided
        let key = options.key.unwrap_or_else(|| {
            let uuid = uuid::Uuid::new_v4();
            format!("{}/{}", chrono::Utc::now().format("%Y/%m/%d"), uuid)
        });

        // Detect content type
        let content_type = options
            .content_type
            .unwrap_or_else(|| "application/octet-stream".to_string());

        // Calculate SHA256
        let mut hasher = Sha256::new();
        hasher.update(data);
        let sha256 = hex::encode(hasher.finalize());

        // Upload
        let response = bucket
            .put_object_with_content_type(&key, data, &content_type)
            .await
            .map_err(|e| StorageError::Upload(e.to_string()))?;

        let etag = response
            .headers()
            .get("etag")
            .map(|v| v.to_string())
            .unwrap_or_default()
            .trim_matches('"')
            .to_string();

        // Generate URL
        let url = self.get_object_url(bucket_name, &key);

        info!(
            bucket = bucket_name,
            key = key,
            size = data.len(),
            "Object uploaded"
        );

        Ok(UploadResult {
            key,
            bucket: bucket_name.to_string(),
            etag,
            size: data.len() as u64,
            content_type,
            url,
            sha256: Some(sha256),
        })
    }

    /// Upload a file from disk
    pub async fn upload_file(
        &self,
        bucket_name: &str,
        file_path: impl AsRef<Path>,
        options: UploadOptions,
    ) -> StorageResult<UploadResult> {
        let path = file_path.as_ref();

        // Read file
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| StorageError::InvalidFile(e.to_string()))?;

        // Auto-detect content type from extension
        let content_type = options.content_type.clone().or_else(|| {
            mime_guess::from_path(path)
                .first()
                .map(|m| m.to_string())
        });

        // Use filename as part of key if not specified
        let options = if options.key.is_none() {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            let uuid = uuid::Uuid::new_v4();
            let key = format!(
                "{}/{}/{}",
                chrono::Utc::now().format("%Y/%m/%d"),
                uuid,
                filename
            );
            UploadOptions {
                key: Some(key),
                content_type,
                ..options
            }
        } else {
            UploadOptions {
                content_type,
                ..options
            }
        };

        self.upload_bytes(bucket_name, &data, options).await
    }

    /// Upload with a stream (for large files)
    pub async fn upload_stream<S>(
        &self,
        bucket_name: &str,
        key: &str,
        stream: S,
        content_length: u64,
        content_type: &str,
    ) -> StorageResult<UploadResult>
    where
        S: futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Unpin + Send + 'static,
    {
        let bucket = self.create_bucket(bucket_name)?;

        // For streaming, we need to use multipart upload for large files
        // For now, collect stream and upload
        use futures::StreamExt;

        let mut data = Vec::with_capacity(content_length as usize);
        let mut stream = stream;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(StorageError::Io)?;
            data.extend_from_slice(&chunk);
        }

        self.upload_bytes(
            bucket_name,
            &data,
            UploadOptions::new()
                .with_key(key)
                .with_content_type(content_type),
        )
        .await
    }

    /// Download object as bytes
    pub async fn download_bytes(&self, bucket_name: &str, key: &str) -> StorageResult<Vec<u8>> {
        let bucket = self.create_bucket(bucket_name)?;

        let response = bucket
            .get_object(key)
            .await
            .map_err(|e| match &e {
                s3::error::S3Error::HttpFailWithBody(404, _) => StorageError::NotFound {
                    bucket: bucket_name.to_string(),
                    key: key.to_string(),
                },
                _ => StorageError::Download(e.to_string()),
            })?;

        Ok(response.to_vec())
    }

    /// Download object to a file
    pub async fn download_file(
        &self,
        bucket_name: &str,
        key: &str,
        file_path: impl AsRef<Path>,
    ) -> StorageResult<()> {
        let data = self.download_bytes(bucket_name, key).await?;
        tokio::fs::write(file_path, data).await?;
        Ok(())
    }

    /// Get object metadata
    pub async fn get_object_info(&self, bucket_name: &str, key: &str) -> StorageResult<ObjectInfo> {
        let bucket = self.create_bucket(bucket_name)?;

        let (head, _) = bucket.head_object(key).await.map_err(|e| match &e {
            s3::error::S3Error::HttpFailWithBody(404, _) => StorageError::NotFound {
                bucket: bucket_name.to_string(),
                key: key.to_string(),
            },
            _ => StorageError::S3(e.to_string()),
        })?;

        Ok(ObjectInfo {
            key: key.to_string(),
            bucket: bucket_name.to_string(),
            content_type: head.content_type.unwrap_or_default(),
            size: head.content_length.unwrap_or(0) as u64,
            etag: head.e_tag,
            last_modified: head.last_modified.and_then(|s| {
                chrono::DateTime::parse_from_rfc2822(&s)
                    .ok()
                    .map(|dt| dt.with_timezone(&chrono::Utc))
            }),
            metadata: HashMap::new(),
        })
    }

    /// Check if an object exists
    pub async fn object_exists(&self, bucket_name: &str, key: &str) -> StorageResult<bool> {
        match self.get_object_info(bucket_name, key).await {
            Ok(_) => Ok(true),
            Err(StorageError::NotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    /// Delete an object
    pub async fn delete_object(&self, bucket_name: &str, key: &str) -> StorageResult<()> {
        let bucket = self.create_bucket(bucket_name)?;

        bucket
            .delete_object(key)
            .await
            .map_err(|e| StorageError::Delete(e.to_string()))?;

        info!(bucket = bucket_name, key = key, "Object deleted");
        Ok(())
    }

    /// Delete multiple objects
    pub async fn delete_objects(&self, bucket_name: &str, keys: &[String]) -> StorageResult<()> {
        for key in keys {
            self.delete_object(bucket_name, key).await?;
        }
        Ok(())
    }

    /// List objects in a bucket
    pub async fn list_objects(
        &self,
        bucket_name: &str,
        prefix: Option<&str>,
        max_keys: Option<usize>,
    ) -> StorageResult<ListObjectsResult> {
        let bucket = self.create_bucket(bucket_name)?;

        let results = bucket
            .list(prefix.unwrap_or("").to_string(), None)
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;

        let mut objects = Vec::new();
        let mut prefixes = Vec::new();

        for result in results {
            for object in result.contents {
                if let Some(max) = max_keys {
                    if objects.len() >= max {
                        break;
                    }
                }

                // Parse last_modified timestamp - it's a String in rust-s3 0.35
                let last_modified: Option<chrono::DateTime<chrono::Utc>> =
                    chrono::DateTime::parse_from_rfc3339(&object.last_modified)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc));

                objects.push(ObjectInfo {
                    key: object.key.clone(),
                    bucket: bucket_name.to_string(),
                    content_type: String::new(),
                    size: object.size,
                    etag: object.e_tag.clone(),
                    last_modified,
                    metadata: HashMap::new(),
                });
            }

            if let Some(common_prefixes) = result.common_prefixes {
                for prefix_item in common_prefixes {
                    // prefix is String in rust-s3 0.35
                    prefixes.push(prefix_item.prefix);
                }
            }
        }

        Ok(ListObjectsResult {
            objects,
            prefixes,
            is_truncated: false,
            continuation_token: None,
        })
    }

    /// Generate a presigned URL for downloading
    pub async fn get_presigned_download_url(
        &self,
        bucket_name: &str,
        key: &str,
        options: PresignedUrlOptions,
    ) -> StorageResult<String> {
        let bucket = self.create_bucket(bucket_name)?;

        let url = bucket
            .presign_get(key, options.expires_in, None)
            .await
            .map_err(|e| StorageError::PresignedUrl(e.to_string()))?;

        Ok(url)
    }

    /// Generate a presigned URL for uploading
    pub async fn get_presigned_upload_url(
        &self,
        bucket_name: &str,
        key: &str,
        expires_in: u32,
        content_type: Option<&str>,
    ) -> StorageResult<String> {
        let bucket = self.create_bucket(bucket_name)?;

        let mut custom_headers = HashMap::new();
        if let Some(ct) = content_type {
            custom_headers.insert("Content-Type".to_string(), ct.to_string());
        }

        let url = bucket
            .presign_put(key, expires_in, None, None)
            .await
            .map_err(|e| StorageError::PresignedUrl(e.to_string()))?;

        Ok(url)
    }

    /// Get the direct URL for an object (for public objects)
    pub fn get_object_url(&self, bucket_name: &str, key: &str) -> Option<String> {
        if let Some(public_url) = &self.config.public_url {
            Some(format!("{}/{}/{}", public_url, bucket_name, key))
        } else if self.config.path_style {
            Some(format!("{}/{}/{}", self.config.endpoint, bucket_name, key))
        } else {
            None
        }
    }

    /// Copy an object
    pub async fn copy_object(
        &self,
        source_bucket: &str,
        source_key: &str,
        dest_bucket: &str,
        dest_key: &str,
    ) -> StorageResult<()> {
        let bucket = self.create_bucket(dest_bucket)?;

        let source = format!("{}/{}", source_bucket, source_key);

        bucket
            .copy_object_internal(&source, dest_key)
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;

        info!(
            source_bucket = source_bucket,
            source_key = source_key,
            dest_bucket = dest_bucket,
            dest_key = dest_key,
            "Object copied"
        );

        Ok(())
    }

    /// Get storage configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config() {
        let config = StorageConfig::rustfs("http://localhost:9000", "access", "secret");
        assert!(config.path_style);
        assert_eq!(config.region, "us-east-1");
    }
}
