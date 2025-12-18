use common::models::{Job, JobStatus};
use common::{Error, Result};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct JobRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct Row {
    id: Uuid,
    job_type: String,
    payload: serde_json::Value,
    status: String,
    result: Option<serde_json::Value>,
    error: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Row {
    fn into_job(self) -> Result<Job> {
        Ok(Job {
            id: self.id,
            job_type: self.job_type,
            payload: self.payload,
            status: serde_json::from_str(&self.status)?,
            result: self.result,
            error: self.error,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl JobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, job: &Job) -> Result<Job> {
        sqlx::query_as::<_, Row>(
            "INSERT INTO jobs (id, job_type, payload, status, result, error, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, job_type, payload, status, result, error, created_at, updated_at",
        )
        .bind(job.id)
        .bind(&job.job_type)
        .bind(&job.payload)
        .bind(serde_json::to_string(&job.status)?)
        .bind(&job.result)
        .bind(&job.error)
        .bind(job.created_at)
        .bind(job.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
        .into_job()
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Job>> {
        match sqlx::query_as::<_, Row>(
            "SELECT id, job_type, payload, status, result, error, created_at, updated_at FROM jobs WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
        {
            Some(row) => Ok(Some(row.into_job()?)),
            None => Ok(None),
        }
    }

    pub async fn update_status(&self, id: &Uuid, status: JobStatus) -> Result<()> {
        sqlx::query("UPDATE jobs SET status = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(serde_json::to_string(&status)?)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn complete(&self, id: &Uuid, result: serde_json::Value) -> Result<()> {
        sqlx::query("UPDATE jobs SET status = $2, result = $3, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(serde_json::to_string(&JobStatus::Completed)?)
            .bind(&result)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn fail(&self, id: &Uuid, error: &str) -> Result<()> {
        sqlx::query("UPDATE jobs SET status = $2, error = $3, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(serde_json::to_string(&JobStatus::Failed)?)
            .bind(error)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }
}
