use crate::state::AppState;
use axum::{extract::State, http::StatusCode, Json};
use deadpool_redis::redis::cmd;
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

pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".into(),
        version: env!("CARGO_PKG_VERSION").into(),
    })
}

pub async fn readiness_check(
    State(state): State<AppState>,
) -> Result<Json<ReadinessResponse>, StatusCode> {
    let db_status = match state.db_pool.health_check() {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let redis_status = match state.redis_pool.get().await {
        Ok(mut conn) => {
            let ping: Result<String, _> = cmd("PING").query_async(&mut *conn).await;
            if ping.is_ok() {
                "connected"
            } else {
                "disconnected"
            }
        }
        Err(_) => "disconnected",
    };

    let all_healthy = db_status == "connected" && redis_status == "connected";

    let response = ReadinessResponse {
        status: if all_healthy { "ready" } else { "not_ready" }.into(),
        database: db_status.into(),
        redis: redis_status.into(),
    };

    if all_healthy {
        Ok(Json(response))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}
