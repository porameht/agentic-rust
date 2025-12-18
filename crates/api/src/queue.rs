//! Job queue producer for API service.
//!
//! This module handles pushing jobs to Redis queue for worker consumption.

use common::models::{EmbedDocumentJob, IndexDocumentJob, ProcessChatJob};
use common::queue::{keys, queues, JobResult, RESULT_TTL_SECONDS};
use common::{Error, Result};
use redis::AsyncCommands;
use uuid::Uuid;

/// Job producer for pushing jobs to Redis queue
#[derive(Clone)]
pub struct JobProducer {
    client: redis::Client,
}

impl JobProducer {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }

    /// Push a chat processing job to the queue
    pub async fn push_chat_job(&self, job: &ProcessChatJob) -> Result<Uuid> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        let job_json = serde_json::to_string(job)?;

        // Push job to queue
        conn.lpush::<_, _, ()>(queues::CHAT_QUEUE, &job_json)
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        // Set initial status
        let status = JobResult::pending(job.job_id);
        let status_json = serde_json::to_string(&status)?;
        conn.set_ex::<_, _, ()>(
            keys::job_status(&job.job_id),
            &status_json,
            RESULT_TTL_SECONDS,
        )
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

        tracing::info!(job_id = %job.job_id, "Chat job pushed to queue");
        Ok(job.job_id)
    }

    /// Push a document embedding job to the queue
    pub async fn push_embed_job(&self, job: &EmbedDocumentJob) -> Result<Uuid> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        let job_json = serde_json::to_string(job)?;

        conn.lpush::<_, _, ()>(queues::EMBED_QUEUE, &job_json)
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        let status = JobResult::pending(job.job_id);
        let status_json = serde_json::to_string(&status)?;
        conn.set_ex::<_, _, ()>(
            keys::job_status(&job.job_id),
            &status_json,
            RESULT_TTL_SECONDS,
        )
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

        tracing::info!(job_id = %job.job_id, "Embed job pushed to queue");
        Ok(job.job_id)
    }

    /// Push a document indexing job to the queue
    pub async fn push_index_job(&self, job: &IndexDocumentJob) -> Result<Uuid> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        let job_json = serde_json::to_string(job)?;

        conn.lpush::<_, _, ()>(queues::INDEX_QUEUE, &job_json)
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        let status = JobResult::pending(job.job_id);
        let status_json = serde_json::to_string(&status)?;
        conn.set_ex::<_, _, ()>(
            keys::job_status(&job.job_id),
            &status_json,
            RESULT_TTL_SECONDS,
        )
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

        tracing::info!(job_id = %job.job_id, "Index job pushed to queue");
        Ok(job.job_id)
    }

    /// Get job status/result from Redis
    pub async fn get_job_status(&self, job_id: &Uuid) -> Result<Option<JobResult>> {
        let mut conn = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        let result: Option<String> = conn
            .get(keys::job_status(job_id))
            .await
            .map_err(|e| Error::Queue(e.to_string()))?;

        match result {
            Some(json) => {
                let status: JobResult = serde_json::from_str(&json)?;
                Ok(Some(status))
            }
            None => Ok(None),
        }
    }
}
