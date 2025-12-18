//! Job models for background processing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// A background job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Job to embed a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedDocumentJob {
    pub job_id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub metadata: serde_json::Value,
}

/// Job to process a chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChatJob {
    pub job_id: Uuid,
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub agent_id: Option<String>,
}

/// Job to index a document into the vector store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocumentJob {
    pub job_id: Uuid,
    pub document_id: Uuid,
}

impl Job {
    pub fn new(job_type: impl Into<String>, payload: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            job_type: job_type.into(),
            payload,
            status: JobStatus::Pending,
            result: None,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }
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
}

impl ProcessChatJob {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            message: message.into(),
            conversation_id: None,
            agent_id: None,
        }
    }

    pub fn with_conversation(mut self, conversation_id: Uuid) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }
}
