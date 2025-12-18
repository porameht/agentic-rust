//! Common types, traits, and utilities shared across the agentic-rust workspace.

pub mod config;
pub mod error;
pub mod models;

pub use config::AppConfig;
pub use error::{Error, Result};
