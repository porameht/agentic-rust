use crate::queue::{JobProducer, RedisPool};
use db::DbPool;
use storage::StorageClient;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub redis_pool: RedisPool,
    pub job_producer: JobProducer,
    pub storage_client: StorageClient,
}

impl AppState {
    pub fn new(db_pool: DbPool, redis_pool: RedisPool, storage_client: StorageClient) -> Self {
        let job_producer = JobProducer::new(redis_pool.clone());
        Self {
            db_pool,
            redis_pool,
            job_producer,
            storage_client,
        }
    }
}
