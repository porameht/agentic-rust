//! Chat endpoints for conversational AI.

use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use common::models::ProcessChatJob;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Chat request body
#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub agent_id: Option<String>,
}

/// Synchronous chat response
#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub conversation_id: Uuid,
    pub sources: Vec<DocumentSource>,
}

/// Document source reference
#[derive(Debug, Serialize)]
pub struct DocumentSource {
    pub document_id: Uuid,
    pub title: String,
    pub relevance_score: f32,
}

/// Async chat response (job created)
#[derive(Debug, Serialize)]
pub struct AsyncChatResponse {
    pub job_id: Uuid,
    pub status: String,
}

/// Job status response
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// Synchronous chat handler
/// Processes the chat request immediately and returns the response
pub async fn chat_handler(
    State(_state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    // TODO: Implement synchronous chat processing
    // 1. Get or create conversation
    // 2. Retrieve relevant documents
    // 3. Generate response with RAG agent
    // 4. Save conversation history
    // 5. Return response with sources

    // Placeholder response
    Ok(Json(ChatResponse {
        response: format!("Processing: {}", request.message),
        conversation_id: request.conversation_id.unwrap_or_else(Uuid::new_v4),
        sources: vec![],
    }))
}

/// Async chat handler
/// Queues the chat request for background processing
pub async fn chat_async_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<AsyncChatResponse>, StatusCode> {
    // Create job with optional conversation and agent
    let mut job = ProcessChatJob::new(&request.message);
    if let Some(conv_id) = request.conversation_id {
        job = job.with_conversation(conv_id);
    }
    if let Some(agent_id) = request.agent_id {
        job = job.with_agent(agent_id);
    }

    // Push job to Redis queue
    let job_id = state.job_producer.push_chat_job(&job).await.map_err(|e| {
        tracing::error!(error = %e, "Failed to push chat job to queue");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(AsyncChatResponse {
        job_id,
        status: "queued".to_string(),
    }))
}

/// Get job status
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    // Fetch job status from Redis
    let result = state
        .job_producer
        .get_job_status(&job_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get job status");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    match result {
        Some(job_result) => Ok(Json(JobStatusResponse {
            job_id: job_result.job_id,
            status: format!("{:?}", job_result.status).to_lowercase(),
            result: job_result.result,
            error: job_result.error,
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}
