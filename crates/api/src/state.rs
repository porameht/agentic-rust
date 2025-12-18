//! Application state shared across handlers.

use db::DbPool;
use std::sync::Arc;

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub redis_client: redis::Client,
    // Add more shared state as needed:
    // pub qdrant_client: QdrantClient,
    // pub agent_registry: Arc<AgentRegistry>,
}

impl AppState {
    pub fn new(db_pool: DbPool, redis_client: redis::Client) -> Self {
        Self {
            db_pool,
            redis_client,
        }
    }
}
