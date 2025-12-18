//! Embed document job for generating document embeddings.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job to embed a document's content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedDocumentJob {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
}

impl EmbedDocumentJob {
    pub fn new(document_id: Uuid, content: impl Into<String>) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            document_id,
            content: content.into(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}
