//! API route definitions.

pub mod chat;
pub mod documents;
pub mod health;

use crate::state::AppState;
use axum::{routing::get, routing::post, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        .route("/ready", get(health::readiness_check))
        // API v1
        .nest("/api/v1", api_v1_routes())
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// API v1 routes
fn api_v1_routes() -> Router<AppState> {
    Router::new()
        // Chat endpoints
        .route("/chat", post(chat::chat_handler))
        .route("/chat/async", post(chat::chat_async_handler))
        .route("/chat/jobs/:job_id", get(chat::get_job_status))
        // Document endpoints
        .route("/documents", post(documents::create_document))
        .route("/documents", get(documents::list_documents))
        .route("/documents/:id", get(documents::get_document))
        .route("/documents/:id", axum::routing::delete(documents::delete_document))
        .route("/documents/:id/index", post(documents::index_document))
        .route("/documents/search", post(documents::search_documents))
}
