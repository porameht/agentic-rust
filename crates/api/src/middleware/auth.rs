//! Authentication middleware.

use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// Simple API key authentication middleware
pub async fn api_key_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get API key from header
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    // TODO: Validate API key against stored keys
    // For now, skip validation if no key is required
    if api_key.is_some() {
        // Validate key here
    }

    Ok(next.run(request).await)
}
