//! Document and chunk models for RAG.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A document in the knowledge base
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A chunk of a document for embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub chunk_index: usize,
    pub metadata: serde_json::Value,
}

/// A document chunk with its embedding vector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddedChunk {
    pub chunk: DocumentChunk,
    pub embedding: Vec<f32>,
}

/// Search result from vector similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: DocumentChunk,
    pub score: f32,
}

impl Document {
    pub fn new(title: impl Into<String>, content: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            content: content.into(),
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl DocumentChunk {
    pub fn new(document_id: Uuid, content: impl Into<String>, chunk_index: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            document_id,
            content: content.into(),
            chunk_index,
            metadata: serde_json::json!({}),
        }
    }
}
