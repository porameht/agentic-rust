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
    pub message: String,
}

/// Job status response
#[derive(Debug, Serialize)]
pub struct JobStatusResponse {
    pub job_id: Uuid,
    pub status: String,
    pub result: Option<ChatResponse>,
    pub error: Option<String>,
}

/// Synchronous chat handler
/// Processes the chat request immediately and returns the response
pub async fn chat_handler(
    State(state): State<AppState>,
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
    let job = ProcessChatJob::new(&request.message);
    let job_id = job.job_id;

    // TODO: Push job to Redis queue using apalis
    // state.job_queue.push(job).await?;

    Ok(Json(AsyncChatResponse {
        job_id,
        message: "Chat request queued for processing".to_string(),
    }))
}

/// Get job status
pub async fn get_job_status(
    State(state): State<AppState>,
    Path(job_id): Path<Uuid>,
) -> Result<Json<JobStatusResponse>, StatusCode> {
    // TODO: Fetch job status from Redis or database

    Ok(Json(JobStatusResponse {
        job_id,
        status: "pending".to_string(),
        result: None,
        error: None,
    }))
}
