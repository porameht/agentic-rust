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

/// Create document request
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
    pub content: String,
    pub metadata: Option<serde_json::Value>,
}

/// Document response
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// List documents query parameters
#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Search documents request
#[derive(Debug, Deserialize)]
pub struct SearchDocumentsRequest {
    pub query: String,
    pub limit: Option<usize>,
}

/// Search result response
#[derive(Debug, Serialize)]
pub struct SearchResultResponse {
    pub document_id: Uuid,
    pub title: String,
    pub content_snippet: String,
    pub score: f32,
}

/// Create a new document
pub async fn create_document(
    State(state): State<AppState>,
    Json(request): Json<CreateDocumentRequest>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let document = Document::new(&request.title, &request.content)
        .with_metadata(request.metadata.unwrap_or_default());

    let repo = DocumentRepository::new(state.db_pool.inner().clone());

    match repo.create(&document).await {
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

/// Get a document by ID
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentResponse>, StatusCode> {
    let repo = DocumentRepository::new(state.db_pool.inner().clone());

    match repo.get(&id).await {
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

/// List all documents
pub async fn list_documents(
    State(state): State<AppState>,
    Query(query): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<DocumentResponse>>, StatusCode> {
    let repo = DocumentRepository::new(state.db_pool.inner().clone());
    let limit = query.limit.unwrap_or(10);
    let offset = query.offset.unwrap_or(0);

    match repo.list(limit, offset).await {
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

/// Delete a document
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo = DocumentRepository::new(state.db_pool.inner().clone());

    match repo.delete(&id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Index a document for RAG (creates embeddings)
pub async fn index_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // TODO: Create EmbedDocumentJob and push to queue
    // This will:
    // 1. Fetch the document
    // 2. Chunk the content
    // 3. Generate embeddings
    // 4. Store in vector database

    Ok(Json(serde_json::json!({
        "message": "Document indexing queued",
        "document_id": id
    })))
}

/// Search documents using semantic search
pub async fn search_documents(
    State(state): State<AppState>,
    Json(request): Json<SearchDocumentsRequest>,
) -> Result<Json<Vec<SearchResultResponse>>, StatusCode> {
    // TODO: Implement semantic search
    // 1. Generate embedding for query
    // 2. Search vector database
    // 3. Return results with relevance scores

    Ok(Json(vec![]))
}
