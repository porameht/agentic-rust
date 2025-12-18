//! Health check endpoints.

use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: String,
    pub database: String,
    pub redis: String,
}

/// Basic health check - always returns OK if server is running
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Readiness check - verifies all dependencies are available
pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<ReadinessResponse>, StatusCode> {
    // Check database
    let db_status = match state.db_pool.health_check().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    // Check Redis
    let redis_status = match state.redis_client.get_connection() {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let all_healthy = db_status == "connected" && redis_status == "connected";

    let response = ReadinessResponse {
        status: if all_healthy { "ready" } else { "not_ready" }.to_string(),
        database: db_status.to_string(),
        redis: redis_status.to_string(),
    };

    if all_healthy {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
