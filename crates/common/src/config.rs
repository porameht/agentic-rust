//! Application configuration management.

use serde::Deserialize;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub qdrant: QdrantConfig,
    pub llm: LlmConfig,
    pub rag: RagConfig,
    pub worker: WorkerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    #[serde(default = "default_collection")]
    pub collection: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_provider")]
    pub provider: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RagConfig {
    #[serde(default = "default_chunk_size")]
    pub chunk_size: usize,
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: usize,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default = "default_similarity_threshold")]
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkerConfig {
    #[serde(default = "default_concurrency")]
    pub concurrency: usize,
}

// Default value functions
fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_max_connections() -> u32 {
    10
}

fn default_collection() -> String {
    "documents".to_string()
}

fn default_provider() -> String {
    "openai".to_string()
}

fn default_model() -> String {
    "gpt-4".to_string()
}

fn default_embedding_model() -> String {
    "text-embedding-3-small".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_chunk_size() -> usize {
    1000
}

fn default_chunk_overlap() -> usize {
    200
}

fn default_top_k() -> usize {
    5
}

fn default_similarity_threshold() -> f32 {
    0.7
}

fn default_concurrency() -> usize {
    4
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> crate::Result<Self> {
        dotenvy::dotenv().ok();

        let config = config::Config::builder()
            .add_source(config::Environment::default().separator("__"))
            .build()
            .map_err(|e| crate::Error::Config(e.to_string()))?;

        config
            .try_deserialize()
            .map_err(|e| crate::Error::Config(e.to_string()))
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            chunk_size: default_chunk_size(),
            chunk_overlap: default_chunk_overlap(),
            top_k: default_top_k(),
            similarity_threshold: default_similarity_threshold(),
        }
    }
}
