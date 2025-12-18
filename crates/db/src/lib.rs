//! Database layer for the application.
//!
//! This crate provides:
//! - PostgreSQL connection pool management
//! - Repository pattern for data access
//! - Database migrations

pub mod pool;
pub mod repositories;

pub use pool::DbPool;
