//! Worker service entry point.

use db::DbPool;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker::queue::QueueConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    info!("Starting worker service...");

    // Initialize database pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://agentic:agentic@localhost:5432/agentic".to_string());

    let db_pool = DbPool::new(&database_url, 5).await?;

    // Initialize Redis
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let queue_config = QueueConfig {
        redis_url: redis_url.clone(),
        concurrency: std::env::var("WORKER_CONCURRENCY")
            .unwrap_or_else(|_| "4".to_string())
            .parse()
            .unwrap_or(4),
    };

    info!(
        "Worker configured with concurrency: {}",
        queue_config.concurrency
    );

    // TODO: Set up apalis workers
    // Example with apalis:
    //
    // use apalis::prelude::*;
    // use apalis_redis::RedisStorage;
    //
    // let storage = RedisStorage::connect(&redis_url).await?;
    //
    // // Register job handlers
    // let chat_worker = WorkerBuilder::new("chat-worker")
    //     .layer(TraceLayer::new())
    //     .data(db_pool.clone())
    //     .backend(storage.clone())
    //     .build_fn(process_chat_job);
    //
    // let embed_worker = WorkerBuilder::new("embed-worker")
    //     .layer(TraceLayer::new())
    //     .data(db_pool.clone())
    //     .backend(storage.clone())
    //     .build_fn(process_embed_job);
    //
    // Monitor::new()
    //     .register(chat_worker)
    //     .register(embed_worker)
    //     .run()
    //     .await?;

    info!("Worker service started. Waiting for jobs...");

    // Keep the worker running
    // In production, this would be the apalis monitor
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
        info!("Worker heartbeat");
    }
}
