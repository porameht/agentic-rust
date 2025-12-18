//! Queue management for job processing.

use redis::Client as RedisClient;

/// Queue configuration
pub struct QueueConfig {
    pub redis_url: String,
    pub concurrency: usize,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            concurrency: 4,
        }
    }
}

/// Create a Redis client for queue operations
pub fn create_redis_client(url: &str) -> Result<RedisClient, redis::RedisError> {
    RedisClient::open(url)
}
