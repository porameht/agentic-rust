//! Application state shared across handlers.

use crate::queue::JobProducer;
use db::DbPool;
use storage::StorageClient;

/// Application state containing shared resources
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub redis_client: redis::Client,
    pub job_producer: JobProducer,
    pub storage_client: StorageClient,
    // Add more shared state as needed:
    // pub qdrant_client: QdrantClient,
    // pub agent_registry: Arc<AgentRegistry>,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        redis_client: redis::Client,
        storage_client: StorageClient,
    ) -> Self {
        let job_producer = JobProducer::new(redis_client.clone());
        Self {
            db_pool,
            redis_client,
            job_producer,
            storage_client,
        }
    }
}
