use common::models::{EmbedDocumentJob, IndexDocumentJob, ProcessChatJob};
use common::queue::{keys, queues, JobResult, RESULT_TTL_SECONDS};
use common::{Error, QueueJobStatus, Result};
use deadpool_redis::{redis::AsyncCommands, Config, Connection, Pool, Runtime};
use std::sync::Arc;
use tokio::sync::Semaphore;

pub type RedisPool = Pool;

pub fn create_pool(redis_url: &str) -> Result<RedisPool> {
    let cfg = Config::from_url(redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
        .map_err(|e| Error::Queue(e.to_string()))
}

pub struct WorkerState {
    pub redis_pool: RedisPool,
}

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

    pub async fn start(&self) -> Result<()> {
        let semaphore = Arc::new(Semaphore::new(self.concurrency));
        tracing::info!(concurrency = self.concurrency, "consumer started");

        loop {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let state = self.state.clone();

            tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) = process_next_job(&state).await {
                    tracing::error!(error = %e, "job failed");
                }
            });

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }
}

async fn conn(state: &WorkerState) -> Result<Connection> {
    state.redis_pool.get().await.map_err(|e| Error::Queue(e.to_string()))
}

async fn set_status(conn: &mut Connection, job_id: uuid::Uuid, status: &JobResult) -> Result<()> {
    let json = serde_json::to_string(status)?;
    conn.set_ex::<_, _, ()>(keys::job_status(&job_id), &json, RESULT_TTL_SECONDS)
        .await
        .map_err(|e| Error::Queue(e.to_string()))
}

async fn process_next_job(state: &WorkerState) -> Result<()> {
    let mut c = conn(state).await?;

    let result: Option<(String, String)> = c
        .brpop(&[queues::CHAT_QUEUE, queues::EMBED_QUEUE, queues::INDEX_QUEUE], 1.0)
        .await
        .map_err(|e| Error::Queue(e.to_string()))?;

    if let Some((queue, job_json)) = result {
        match queue.as_str() {
            q if q == queues::CHAT_QUEUE => {
                process_chat_job(state, serde_json::from_str(&job_json)?).await?;
            }
            q if q == queues::EMBED_QUEUE => {
                process_embed_job(state, serde_json::from_str(&job_json)?).await?;
            }
            q if q == queues::INDEX_QUEUE => {
                process_index_job(state, serde_json::from_str(&job_json)?).await?;
            }
            _ => tracing::warn!(queue, "unknown queue"),
        }
    }
    Ok(())
}

async fn process_chat_job(state: &WorkerState, job: ProcessChatJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, "processing chat");
    let mut c = conn(state).await?;

    set_status(&mut c, job.job_id, &JobResult {
        job_id: job.job_id,
        status: QueueJobStatus::Processing,
        result: None,
        error: None,
        completed_at: None,
    }).await?;

    // TODO: RAG processing
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    set_status(&mut c, job.job_id, &JobResult::completed(
        job.job_id,
        serde_json::json!({
            "response": format!("Processed: {}", job.message),
            "conversation_id": job.conversation_id,
            "sources": []
        }),
    )).await?;

    tracing::info!(job_id = %job.job_id, "chat completed");
    Ok(())
}

async fn process_embed_job(state: &WorkerState, job: EmbedDocumentJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, document_id = %job.document_id, "processing embed");
    let mut c = conn(state).await?;

    // TODO: embedding pipeline
    set_status(&mut c, job.job_id, &JobResult::completed(
        job.job_id,
        serde_json::json!({ "document_id": job.document_id, "chunks_created": 0 }),
    )).await?;

    tracing::info!(job_id = %job.job_id, "embed completed");
    Ok(())
}

async fn process_index_job(state: &WorkerState, job: IndexDocumentJob) -> Result<()> {
    tracing::info!(job_id = %job.job_id, document_id = %job.document_id, "processing index");
    let mut c = conn(state).await?;

    // TODO: indexing pipeline
    set_status(&mut c, job.job_id, &JobResult::completed(
        job.job_id,
        serde_json::json!({ "document_id": job.document_id, "indexed": true }),
    )).await?;

    tracing::info!(job_id = %job.job_id, "index completed");
    Ok(())
}
