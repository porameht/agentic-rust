//! Queue message types for API-Worker communication.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Queue names for different job types
pub mod queues {
    pub const CHAT_QUEUE: &str = "agentic:chat";
    pub const EMBED_QUEUE: &str = "agentic:embed";
    pub const INDEX_QUEUE: &str = "agentic:index";
}

/// Redis keys for job results
pub mod keys {
    use uuid::Uuid;

    pub fn job_result(job_id: &Uuid) -> String {
        format!("agentic:result:{}", job_id)
    }

    pub fn job_status(job_id: &Uuid) -> String {
        format!("agentic:status:{}", job_id)
    }
}

/// Job status stored in Redis
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Result stored in Redis after job completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub status: QueueJobStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl JobResult {
    pub fn pending(job_id: Uuid) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Pending,
            result: None,
            error: None,
            completed_at: None,
        }
    }

    pub fn completed(job_id: Uuid, result: serde_json::Value) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Completed,
            result: Some(result),
            error: None,
            completed_at: Some(chrono::Utc::now()),
        }
    }

    pub fn failed(job_id: Uuid, error: impl Into<String>) -> Self {
        Self {
            job_id,
            status: QueueJobStatus::Failed,
            result: None,
            error: Some(error.into()),
            completed_at: Some(chrono::Utc::now()),
        }
    }
}

/// TTL for job results in Redis (1 hour)
pub const RESULT_TTL_SECONDS: u64 = 3600;
