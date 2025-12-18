//! Job consumer for Worker service.
//!
//! This module handles consuming jobs from Redis queue and processing them.

use common::models::{EmbedDocumentJob, IndexDocumentJob, ProcessChatJob};
use common::queue::{keys, queues, JobResult, RESULT_TTL_SECONDS};
use common::{Error, Result};
use redis::AsyncCommands;
use std::sync::Arc;
use tokio::sync::Semaphore;

/// Worker state shared across job processors
pub struct WorkerState {
    pub redis_client: redis::Client,
    // Add more state as needed:
    // pub db_pool: DbPool,
    // pub qdrant_client: QdrantClient,
    // pub embedding_model: EmbeddingModel,
}

/// Job consumer that polls Redis queues and processes jobs
pub struct JobConsumer {
    state: Arc<WorkerState>,
    concurrency: usize,
}

impl JobConsumer {
    pub fn new(state: WorkerState, concurrency: usize) -> Self {
        Self {
            state: Arc::new(state),
            concurrency,
        }
    }

    /// Start consuming jobs from all queues
    pub async fn start(&self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));

        tracing::info!(
            concurrency = self.concurrency,
            "Starting job consumer"
        );

        loop {
            // Try to acquire a permit
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let state = self.state.clone();

            // Spawn a task to process one job
            tokio::spawn(async move {
                let _permit = permit; // Hold permit until job completes

                if let Err(e) = process_next_job(&state).await {
                    tracing::error!(error = %e, "Error processing job");
                }
            });

            // Small delay to prevent busy-waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

/// Process the next available job from any queue
async fn process_next_job(state: &WorkerState) -> Result<()> {
    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

    // BRPOP from multiple queues with timeout (blocking pop)
    let result: Option<(String, String)> = conn
        .brpop(
            &[queues::CHAT_QUEUE, queues::EMBED_QUEUE, queues::INDEX_QUEUE],
            1.0, // 1 second timeout
        )
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

    if let Some((queue, job_json)) = result {
        match queue.as_str() {
            q if q == queues::CHAT_QUEUE => {
                let job: ProcessChatJob = serde_json::from_str(&job_json)?;
                process_chat_job(state, job).await?;
            }
            q if q == queues::EMBED_QUEUE => {
                let job: EmbedDocumentJob = serde_json::from_str(&job_json)?;
                process_embed_job(state, job).await?;
            }
            q if q == queues::INDEX_QUEUE => {
                let job: IndexDocumentJob = serde_json::from_str(&job_json)?;
                process_index_job(state, job).await?;
            }
            _ => {
                tracing::warn!(queue = %queue, "Unknown queue");
            }
        }
    }

    Ok(())
}

/// Process a chat job
async fn process_chat_job(state: &WorkerState, job: ProcessChatJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, "Processing chat job");

    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

    // Update status to processing
    let processing = JobResult {
        job_id: job.job_id,
        status: common::QueueJobStatus::Processing,
        result: None,
        error: None,
        completed_at: None,
    };
    let status_json = serde_json::to_string(&processing)?;
    conn.set_ex::<_, _, ()>(
        keys::job_status(&job.job_id),
        &status_json,
        RESULT_TTL_SECONDS,
    )
    .await
    .map_err(|e| Error::Queue(e.to_string()))?;

    // TODO: Actual processing with RAG agent
    // 1. Load conversation history
    // 2. Retrieve relevant documents
    // 3. Generate response with LLM
    // 4. Save to database

    // Simulate processing
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let response = format!("Processed: {}", job.message);

    // Store result
    let result = JobResult::completed(
        job.job_id,
        serde_json::json!({
            "response": response,
            "conversation_id": job.conversation_id,
            "sources": []
        }),
    );
    let result_json = serde_json::to_string(&result)?;
    conn.set_ex::<_, _, ()>(
        keys::job_status(&job.job_id),
        &result_json,
        RESULT_TTL_SECONDS,
    )
    .await
    .map_err(|e| Error::Queue(e.to_string()))?;

    tracing::info!(job_id = %job.job_id, "Chat job completed");
    Ok(())
}

/// Process an embed document job
async fn process_embed_job(state: &WorkerState, job: EmbedDocumentJob) -> Result<()> {
    tracing::info!(
        job_id = %job.job_id,
        document_id = %job.document_id,
        "Processing embed job"
    );

    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

    // TODO: Actual embedding
    // 1. Chunk document
    // 2. Generate embeddings
    // 3. Store in vector database

    let result = JobResult::completed(
        job.job_id,
        serde_json::json!({
            "document_id": job.document_id,
            "chunks_created": 0
        }),
    );
    let result_json = serde_json::to_string(&result)?;
    conn.set_ex::<_, _, ()>(
        keys::job_status(&job.job_id),
        &result_json,
        RESULT_TTL_SECONDS,
    )
    .await
    .map_err(|e| Error::Queue(e.to_string()))?;

    tracing::info!(job_id = %job.job_id, "Embed job completed");
    Ok(())
}

/// Process an index document job
async fn process_index_job(state: &WorkerState, job: IndexDocumentJob) -> Result<()> {
    tracing::info!(
        job_id = %job.job_id,
        document_id = %job.document_id,
        "Processing index job"
    );

    let mut conn = state
        .redis_client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

    // TODO: Full indexing pipeline
    // 1. Fetch document from DB
    // 2. Chunk content
    // 3. Generate embeddings
    // 4. Store in vector DB

    let result = JobResult::completed(
        job.job_id,
        serde_json::json!({
            "document_id": job.document_id,
            "indexed": true
        }),
    );
    let result_json = serde_json::to_string(&result)?;
    conn.set_ex::<_, _, ()>(
        keys::job_status(&job.job_id),
        &result_json,
        RESULT_TTL_SECONDS,
    )
    .await
    .map_err(|e| Error::Queue(e.to_string()))?;

    tracing::info!(job_id = %job.job_id, "Index job completed");
    Ok(())
}
