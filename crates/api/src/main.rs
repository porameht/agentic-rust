//! API server entry point.

use api::{create_router, AppState};
use db::DbPool;
use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database pool
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://agentic:agentic@localhost:5432/agentic".to_string());

    let db_pool = DbPool::new(&database_url, 10).await?;

    // Run migrations
    info!("Running database migrations...");
    db_pool.run_migrations().await?;

    // Initialize Redis client
    let redis_url =
        std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string());

    let redis_client = redis::Client::open(redis_url)?;

    // Create application state
    let state = AppState::new(db_pool, redis_client);

    // Create router
    let app = create_router(state);

    // Get server address
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port: u16 = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()?;

    let addr = SocketAddr::new(host.parse()?, port);

    info!("Starting API server on {}", addr);

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
