//! Database connection pool management using Diesel r2d2.

use common::{Error, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::time::Duration;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct DbPool {
    pool: PgPool,
}

impl DbPool {
    pub fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(max_connections)
            .connection_timeout(Duration::from_secs(30))
            .build(manager)
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(Self { pool })
    }

    pub fn conn(&self) -> Result<PgConn> {
        self.pool.get().map_err(|e| Error::Database(e.to_string()))
    }

    pub fn inner(&self) -> &PgPool {
        &self.pool
    }

    pub fn run_migrations(&self) -> Result<()> {
        let mut conn = self.conn()?;
        conn.run_pending_migrations(MIGRATIONS)
            .map_err(|e| Error::Database(e.to_string()))?;
        tracing::info!("Database migrations completed");
        Ok(())
    }

    pub fn health_check(&self) -> Result<()> {
        let _conn = self.conn()?;
        Ok(())
    }
}
