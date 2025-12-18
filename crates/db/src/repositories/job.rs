//! Job repository for tracking background jobs.

use common::models::{Job, JobStatus};
use common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for job operations
pub struct JobRepository {
    pool: PgPool,
}

impl JobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new job
    pub async fn create(&self, job: &Job) -> Result<Job> {
        let status_str = serde_json::to_string(&job.status)?;

        let row = sqlx::query!(
            r#"
            INSERT INTO jobs (id, job_type, payload, status, result, error, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, job_type, payload, status, result, error, created_at, updated_at
            "#,
            job.id,
            job.job_type,
            job.payload,
            status_str,
            job.result,
            job.error,
            job.created_at,
            job.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let status: JobStatus = serde_json::from_str(&row.status)?;

        Ok(Job {
            id: row.id,
            job_type: row.job_type,
            payload: row.payload,
            status,
            result: row.result,
            error: row.error,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Get a job by ID
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<Job>> {
        let row = sqlx::query!(
            r#"
            SELECT id, job_type, payload, status, result, error, created_at, updated_at
            FROM jobs
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let status: JobStatus = serde_json::from_str(&r.status)?;
                Ok(Some(Job {
                    id: r.id,
                    job_type: r.job_type,
                    payload: r.payload,
                    status,
                    result: r.result,
                    error: r.error,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Update job status
    pub async fn update_status(&self, id: &Uuid, status: JobStatus) -> Result<()> {
        let status_str = serde_json::to_string(&status)?;

        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            status_str
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Update job with result
    pub async fn complete(&self, id: &Uuid, result: serde_json::Value) -> Result<()> {
        let status_str = serde_json::to_string(&JobStatus::Completed)?;

        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $2, result = $3, updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            status_str,
            result
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Update job with error
    pub async fn fail(&self, id: &Uuid, error: &str) -> Result<()> {
        let status_str = serde_json::to_string(&JobStatus::Failed)?;

        sqlx::query!(
            r#"
            UPDATE jobs
            SET status = $2, error = $3, updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            status_str,
            error
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }
}
