//! REST API service for the agentic-rust application.
//!
//! This crate provides:
//! - Axum-based HTTP server
//! - Chat endpoints (sync and async)
//! - Document management endpoints
//! - Agent management endpoints

pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod state;

pub use routes::create_router;
pub use state::AppState;
