//! Database layer using Diesel ORM.
//!
//! This crate provides:
//! - PostgreSQL connection pool management with r2d2
//! - Diesel ORM models and schema
//! - Repository pattern for data access
//! - Embedded migrations with auto-run on startup

pub mod models;
pub mod pool;
pub mod repositories;
pub mod schema;

pub use pool::{DbPool, PgConn, PgPool, MIGRATIONS};
pub use repositories::{ConversationRepository, DocumentRepository, JobRepository, MessageRepository};
