//! Document management endpoints.

use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use common::models::Document;
use db::repositories::DocumentRepository;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct SearchDocumentsRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultResponse {
    pub document_id: Uuid,
    pub title: String,
    pub content_snippet: String,
    pub score: f32,
}

pub async fn create_document(
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let document = Document::new(&request.title, &request.content)
        .with_metadata(request.metadata.unwrap_or_default());

    let repo = DocumentRepository::new(state.db_pool.clone());

    match repo.create(&document) {
        Ok(doc) => Ok(Json(DocumentResponse {
            id: doc.id,
            title: doc.title,
            content: doc.content,
            metadata: doc.metadata,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let repo = DocumentRepository::new(state.db_pool.clone());

    match repo.get(&id) {
        Ok(Some(doc)) => Ok(Json(DocumentResponse {
            id: doc.id,
            title: doc.title,
            content: doc.content,
            metadata: doc.metadata,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn list_documents(
    State(state): State<AppState>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<DocumentResponse>>, StatusCode> {
    let repo = DocumentRepository::new(state.db_pool.clone());
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    match repo.list(limit, offset) {
        Ok(docs) => Ok(Json(
            docs.into_iter()
                .map(|doc| DocumentResponse {
                    id: doc.id,
                    title: doc.title,
                    content: doc.content,
                    metadata: doc.metadata,
                    created_at: doc.created_at,
                    updated_at: doc.updated_at,
                })
                .collect(),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = DocumentRepository::new(state.db_pool.clone());

    match repo.delete(&id) {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn index_document(
    State(_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "message": "Document indexing queued",
        "document_id": id
    })))
}

pub async fn search_documents(
    State(_state): State<AppState>,
    Json(_request): Json<SearchDocumentsRequest>,
) -> Result<Json<Vec<SearchResultResponse>>, StatusCode> {
    Ok(Json(vec![]))
}
