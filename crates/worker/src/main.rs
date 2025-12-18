use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use worker::{consumer, JobConsumer, WorkerState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "worker=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis_pool = consumer::create_pool(&redis_url)?;
    info!("Redis pool initialized");

    let concurrency: usize = std::env::var("WORKER_CONCURRENCY")
        .unwrap_or_else(|_| "4".into())
        .parse()
        .unwrap_or(4);

    let state = WorkerState { redis_pool };
    let consumer = JobConsumer::new(state, concurrency);

    info!(concurrency, "worker started");
    consumer.start().await?;

    Ok(())
}
