//! API route definitions.

pub mod brochures;
pub mod chat;
pub mod documents;
pub mod files;
pub mod health;
pub mod products;

use crate::state::AppState;
use axum::{routing::get, routing::post, routing::put, routing::delete, Router};
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
        // Chat endpoints (Sales Agent)
        .route("/chat", post(chat::chat_handler))
        .route("/chat/async", post(chat::chat_async_handler))
        .route("/chat/jobs/:job_id", get(chat::get_job_status))

        // Product endpoints
        .route("/products", get(products::list_products))
        .route("/products", post(products::create_product))
        .route("/products/recommend", post(products::get_recommendations))
        .route("/products/:id", get(products::get_product))
        .route("/products/:id", put(products::update_product))
        .route("/products/:id", delete(products::delete_product))
        .route("/products/:id/index", post(products::index_product))
        .route("/products/:id/images", post(files::upload_product_image))

        // Brochure/Download endpoints
        .route("/brochures", get(brochures::list_brochures))
        .route("/brochures", post(brochures::create_brochure))
        .route("/brochures/:id", get(brochures::get_brochure))
        .route("/brochures/:id", put(brochures::update_brochure))
        .route("/brochures/:id", delete(brochures::delete_brochure))
        .route("/brochures/:id/download", get(brochures::get_download_url))
        .route("/products/:product_id/brochures", get(brochures::get_product_brochures))

        // File/Storage endpoints (RustFS)
        .route("/files/brochures", post(files::upload_brochure))
        .route("/files/:bucket", get(files::list_files))
        .route("/files/:bucket/:key", get(files::get_file_info))
        .route("/files/:bucket/:key", delete(files::delete_file))
        .route("/files/:bucket/:key/download", get(files::get_download_url))
        .route("/files/:bucket/upload-url", get(files::get_upload_url))

        // Document endpoints (Knowledge Base)
        .route("/documents", post(documents::create_document))
        .route("/documents", get(documents::list_documents))
        .route("/documents/:id", get(documents::get_document))
        .route("/documents/:id", delete(documents::delete_document))
        .route("/documents/:id/index", post(documents::index_document))
        .route("/documents/search", post(documents::search_documents))
}
