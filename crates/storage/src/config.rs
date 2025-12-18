use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    #[serde(default = "default_region")]
    pub region: String,
    #[serde(default = "default_path_style")]
    pub path_style: bool,
    #[serde(default)]
    pub default_bucket: Option<String>,
    #[serde(default)]
    pub public_url: Option<String>,
}

fn default_region() -> String {
    "us-east-1".into()
}

fn default_path_style() -> bool {
    true
}

impl StorageConfig {
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

    pub fn rustfs(
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self::new(endpoint, access_key, secret_key).with_path_style(true)
    }

    pub fn minio(
        endpoint: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self::new(endpoint, access_key, secret_key).with_path_style(true)
    }

    pub fn aws_s3(
        region: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
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

    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    pub fn with_path_style(mut self, path_style: bool) -> Self {
        self.path_style = path_style;
        self
    }

    pub fn with_default_bucket(mut self, bucket: impl Into<String>) -> Self {
        self.default_bucket = Some(bucket.into());
        self
    }

    pub fn with_public_url(mut self, url: impl Into<String>) -> Self {
        self.public_url = Some(url.into());
        self
    }

    pub fn from_env() -> crate::StorageResult<Self> {
        Ok(Self {
            endpoint: std::env::var("STORAGE_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".into()),
            access_key: std::env::var("STORAGE_ACCESS_KEY").unwrap_or_else(|_| "admin".into()),
            secret_key: std::env::var("STORAGE_SECRET_KEY")
                .unwrap_or_else(|_| "adminpassword".into()),
            region: std::env::var("STORAGE_REGION").unwrap_or_else(|_| default_region()),
            path_style: std::env::var("STORAGE_PATH_STYLE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
            default_bucket: std::env::var("STORAGE_DEFAULT_BUCKET").ok(),
            public_url: std::env::var("STORAGE_PUBLIC_URL").ok(),
        })
    }
}
