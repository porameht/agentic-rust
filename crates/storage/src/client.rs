use crate::config::StorageConfig;
use crate::error::{StorageError, StorageResult};
use crate::models::{
    ListObjectsResult, ObjectInfo, PresignedUrlOptions, UploadOptions, UploadResult,
};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info};

#[derive(Clone)]
pub struct StorageClient {
    config: StorageConfig,
}

impl StorageClient {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    pub fn from_env() -> StorageResult<Self> {
        Ok(Self::new(StorageConfig::from_env()?))
    }

    fn region(&self) -> Region {
        Region::Custom {
            region: self.config.region.clone(),
            endpoint: self.config.endpoint.clone(),
        }
    }

    fn credentials(&self) -> StorageResult<Credentials> {
        Credentials::new(
            Some(&self.config.access_key),
            Some(&self.config.secret_key),
            None,
            None,
            None,
        )
        .map_err(|e| StorageError::Config(e.to_string()))
    }

    fn bucket(&self, name: &str) -> StorageResult<Box<Bucket>> {
        let mut bucket = Bucket::new(name, self.region(), self.credentials()?)
            .map_err(|e| StorageError::Bucket(e.to_string()))?;

        if self.config.path_style {
            bucket = bucket.with_path_style();
        }
        Ok(bucket)
    }

    pub async fn bucket_exists(&self, name: &str) -> StorageResult<bool> {
        let bucket = self.bucket(name)?;
        match bucket.head_object("/").await {
            Ok(_) => Ok(true),
            Err(s3::error::S3Error::HttpFailWithBody(404, _)) => Ok(false),
            Err(s3::error::S3Error::HttpFailWithBody(403, _)) => Ok(true),
            Err(e) => match bucket.list(String::new(), Some("/".to_string())).await {
                Ok(_) => Ok(true),
                Err(_) => Err(StorageError::Bucket(e.to_string())),
            },
        }
    }

    pub async fn create_bucket_if_not_exists(&self, name: &str) -> StorageResult<()> {
        match Bucket::create_with_path_style(
            name,
            self.region(),
            self.credentials()?,
            s3::BucketConfiguration::default(),
        )
        .await
        {
            Ok(_) => {
                info!(bucket = name, "created");
                Ok(())
            }
            Err(s3::error::S3Error::HttpFailWithBody(409, _)) => {
                debug!(bucket = name, "exists");
                Ok(())
            }
            Err(e) => Err(StorageError::Bucket(e.to_string())),
        }
    }

    pub async fn upload_bytes(
        &self,
        bucket: &str,
        data: &[u8],
        opts: UploadOptions,
    ) -> StorageResult<UploadResult> {
        let b = self.bucket(bucket)?;

        let key = opts.key.unwrap_or_else(|| {
            format!(
                "{}/{}",
                chrono::Utc::now().format("%Y/%m/%d"),
                uuid::Uuid::new_v4()
            )
        });

        let content_type = opts
            .content_type
            .unwrap_or_else(|| "application/octet-stream".into());

        let mut hasher = Sha256::new();
        hasher.update(data);
        let sha256 = hex::encode(hasher.finalize());

        let resp = b
            .put_object_with_content_type(&key, data, &content_type)
            .await
            .map_err(|e| StorageError::Upload(e.to_string()))?;

        let etag = resp
            .headers()
            .get("etag")
            .map(|v| v.trim_matches('"').to_string())
            .unwrap_or_default();

        let url = self.object_url(bucket, &key);
        info!(bucket, key, size = data.len(), "uploaded");

        Ok(UploadResult {
            key,
            bucket: bucket.into(),
            etag,
            size: data.len() as u64,
            content_type,
            url,
            sha256: Some(sha256),
        })
    }

    pub async fn upload_file(
        &self,
        bucket: &str,
        path: impl AsRef<Path>,
        opts: UploadOptions,
    ) -> StorageResult<UploadResult> {
        let path = path.as_ref();
        let data = tokio::fs::read(path)
            .await
            .map_err(|e| StorageError::InvalidFile(e.to_string()))?;

        let content_type = opts
            .content_type
            .clone()
            .or_else(|| mime_guess::from_path(path).first().map(|m| m.to_string()));

        let opts = if opts.key.is_none() {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
            UploadOptions {
                key: Some(format!(
                    "{}/{}/{}",
                    chrono::Utc::now().format("%Y/%m/%d"),
                    uuid::Uuid::new_v4(),
                    filename
                )),
                content_type,
                ..opts
            }
        } else {
            UploadOptions {
                content_type,
                ..opts
            }
        };

        self.upload_bytes(bucket, &data, opts).await
    }

    pub async fn upload_stream<S>(
        &self,
        bucket: &str,
        key: &str,
        stream: S,
        content_length: u64,
        content_type: &str,
    ) -> StorageResult<UploadResult>
    where
        S: futures::Stream<Item = Result<bytes::Bytes, std::io::Error>> + Unpin + Send + 'static,
    {
        use futures::StreamExt;

        let mut data = Vec::with_capacity(content_length as usize);
        let mut stream = stream;
        while let Some(chunk) = stream.next().await {
            data.extend_from_slice(&chunk.map_err(StorageError::Io)?);
        }

        self.upload_bytes(
            bucket,
            &data,
            UploadOptions::new()
                .with_key(key)
                .with_content_type(content_type),
        )
        .await
    }

    pub async fn download_bytes(&self, bucket: &str, key: &str) -> StorageResult<Vec<u8>> {
        self.bucket(bucket)?
            .get_object(key)
            .await
            .map(|r| r.to_vec())
            .map_err(|e| match &e {
                s3::error::S3Error::HttpFailWithBody(404, _) => StorageError::NotFound {
                    bucket: bucket.into(),
                    key: key.into(),
                },
                _ => StorageError::Download(e.to_string()),
            })
    }

    pub async fn download_file(
        &self,
        bucket: &str,
        key: &str,
        path: impl AsRef<Path>,
    ) -> StorageResult<()> {
        let data = self.download_bytes(bucket, key).await?;
        tokio::fs::write(path, data).await?;
        Ok(())
    }

    pub async fn object_info(&self, bucket: &str, key: &str) -> StorageResult<ObjectInfo> {
        let (head, _) = self
            .bucket(bucket)?
            .head_object(key)
            .await
            .map_err(|e| match &e {
                s3::error::S3Error::HttpFailWithBody(404, _) => StorageError::NotFound {
                    bucket: bucket.into(),
                    key: key.into(),
                },
                _ => StorageError::S3(e.to_string()),
            })?;

        Ok(ObjectInfo {
            key: key.into(),
            bucket: bucket.into(),
            content_type: head.content_type.unwrap_or_default(),
            size: head.content_length.unwrap_or(0) as u64,
            etag: head.e_tag,
            last_modified: head
                .last_modified
                .and_then(|s| chrono::DateTime::parse_from_rfc2822(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            metadata: HashMap::new(),
        })
    }

    pub async fn object_exists(&self, bucket: &str, key: &str) -> StorageResult<bool> {
        match self.object_info(bucket, key).await {
            Ok(_) => Ok(true),
            Err(StorageError::NotFound { .. }) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub async fn delete(&self, bucket: &str, key: &str) -> StorageResult<()> {
        self.bucket(bucket)?
            .delete_object(key)
            .await
            .map_err(|e| StorageError::Delete(e.to_string()))?;
        info!(bucket, key, "deleted");
        Ok(())
    }

    pub async fn delete_many(&self, bucket: &str, keys: &[String]) -> StorageResult<()> {
        for key in keys {
            self.delete(bucket, key).await?;
        }
        Ok(())
    }

    pub async fn list(
        &self,
        bucket: &str,
        prefix: Option<&str>,
        max: Option<usize>,
    ) -> StorageResult<ListObjectsResult> {
        let results = self
            .bucket(bucket)?
            .list(prefix.unwrap_or("").into(), None)
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;

        let mut objects = Vec::new();
        let mut prefixes = Vec::new();

        for result in results {
            for obj in result.contents {
                if max.is_some_and(|m| objects.len() >= m) {
                    break;
                }
                objects.push(ObjectInfo {
                    key: obj.key.clone(),
                    bucket: bucket.into(),
                    content_type: String::new(),
                    size: obj.size,
                    etag: obj.e_tag.clone(),
                    last_modified: chrono::DateTime::parse_from_rfc3339(&obj.last_modified)
                        .ok()
                        .map(|dt| dt.with_timezone(&chrono::Utc)),
                    metadata: HashMap::new(),
                });
            }
            if let Some(cp) = result.common_prefixes {
                prefixes.extend(cp.into_iter().map(|p| p.prefix));
            }
        }

        Ok(ListObjectsResult {
            objects,
            prefixes,
            is_truncated: false,
            continuation_token: None,
        })
    }

    pub async fn presigned_download(
        &self,
        bucket: &str,
        key: &str,
        opts: PresignedUrlOptions,
    ) -> StorageResult<String> {
        self.bucket(bucket)?
            .presign_get(key, opts.expires_in, None)
            .await
            .map_err(|e| StorageError::PresignedUrl(e.to_string()))
    }

    pub async fn presigned_upload(
        &self,
        bucket: &str,
        key: &str,
        expires_in: u32,
        _content_type: Option<&str>,
    ) -> StorageResult<String> {
        self.bucket(bucket)?
            .presign_put(key, expires_in, None, None)
            .await
            .map_err(|e| StorageError::PresignedUrl(e.to_string()))
    }

    pub fn object_url(&self, bucket: &str, key: &str) -> Option<String> {
        self.config
            .public_url
            .as_ref()
            .map(|url| format!("{}/{}/{}", url, bucket, key))
            .or_else(|| {
                self.config
                    .path_style
                    .then(|| format!("{}/{}/{}", self.config.endpoint, bucket, key))
            })
    }

    pub async fn copy(
        &self,
        src_bucket: &str,
        src_key: &str,
        dst_bucket: &str,
        dst_key: &str,
    ) -> StorageResult<()> {
        self.bucket(dst_bucket)?
            .copy_object_internal(&format!("{}/{}", src_bucket, src_key), dst_key)
            .await
            .map_err(|e| StorageError::S3(e.to_string()))?;
        info!(src_bucket, src_key, dst_bucket, dst_key, "copied");
        Ok(())
    }

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
