use api::{create_router, queue, AppState};
use db::DbPool;
use std::net::SocketAddr;
use storage::{StorageClient, StorageConfig};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug,storage=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://agentic:agentic@localhost:5432/agentic".into());
    let db_pool = DbPool::new(&database_url, 10)?;

    info!("Running migrations...");
    db_pool.run_migrations()?;

    let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());
    let redis_pool = queue::create_pool(&redis_url)?;
    info!("Redis pool initialized");

    let storage_config = StorageConfig::from_env().unwrap_or_else(|_| {
        StorageConfig::rustfs("http://localhost:9000", "admin", "adminpassword")
            .with_default_bucket("brochures")
    });
    let storage_client = StorageClient::new(storage_config);
    info!("Storage initialized");

    for bucket in &["brochures", "products", "documents"] {
        if let Err(e) = storage_client.create_bucket_if_not_exists(bucket).await {
            tracing::warn!(bucket, error = %e, "bucket creation failed");
        }
    }

    let state = AppState::new(db_pool, redis_pool, storage_client);
    let app = create_router(state);

    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".into());
    let port: u16 = std::env::var("SERVER_PORT").unwrap_or_else(|_| "8080".into()).parse()?;
    let addr = SocketAddr::new(host.parse()?, port);

    info!("API server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
