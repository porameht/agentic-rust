//! Worker service entry point.

use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker::{JobConsumer, WorkerState};

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

    // Initialize Redis client
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let redis_client = redis::Client::open(redis_url)?;

    // Get concurrency from environment
    let concurrency: usize = std::env::var("WORKER_CONCURRENCY")
        .unwrap_or_else(|_| "4".to_string())
        .parse()
        .unwrap_or(4);

    info!(concurrency = concurrency, "Worker configured");

    // Create worker state
    let state = WorkerState { redis_client };

    // Create and start job consumer
    let consumer = JobConsumer::new(state, concurrency);

    info!("Worker service started. Waiting for jobs...");

    // Start consuming jobs (this blocks forever)
    consumer.start().await?;

    Ok(())
}
