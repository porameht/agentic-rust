//! Database connection pool management.

use common::{Error, Result};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database connection pool wrapper
#[derive(Clone)]
pub struct DbPool {
    pool: PgPool,
}

impl DbPool {
    /// Create a new database pool from connection string
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(30))
            .connect(database_url)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    /// Get a reference to the underlying pool
    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    /// Run database migrations
    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("../../migrations")
            .run(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Check database connectivity
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }
}
