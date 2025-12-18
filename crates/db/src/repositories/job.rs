//! Job repository using Diesel ORM.

use crate::models::{Job as DbJob, NewJob};
use crate::pool::DbPool;
use crate::schema::jobs;
use chrono::Utc;
use common::models::{Job, JobStatus};
use common::{Error, Result};
use diesel::prelude::*;
use uuid::Uuid;

pub struct JobRepository {
    pool: DbPool,
}

impl JobRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, job: &Job) -> Result<Job> {
        let mut conn = self.pool.conn()?;
        let new_job = NewJob {
            id: job.id,
            job_type: &job.job_type,
            payload: job.payload.clone(),
            status: &serde_json::to_string(&job.status)?,
            result: job.result.clone(),
            error: job.error.as_deref(),
            created_at: job.created_at,
            updated_at: job.updated_at,
        };

        let row: DbJob = diesel::insert_into(jobs::table)
            .values(&new_job)
            .returning(DbJob::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        Self::row_to_job(row)
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Job>> {
        let mut conn = self.pool.conn()?;
        let row: Option<DbJob> = jobs::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Self::row_to_job(r)?)),
            None => Ok(None),
        }
    }

    pub fn update_status(&self, id: &Uuid, status: JobStatus) -> Result<()> {
        let mut conn = self.pool.conn()?;
        diesel::update(jobs::table.find(id))
            .set((
                jobs::status.eq(serde_json::to_string(&status)?),
                jobs::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub fn complete(&self, id: &Uuid, result: serde_json::Value) -> Result<()> {
        let mut conn = self.pool.conn()?;
        diesel::update(jobs::table.find(id))
            .set((
                jobs::status.eq(serde_json::to_string(&JobStatus::Completed)?),
                jobs::result.eq(Some(result)),
                jobs::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub fn fail(&self, id: &Uuid, error: &str) -> Result<()> {
        let mut conn = self.pool.conn()?;
        diesel::update(jobs::table.find(id))
            .set((
                jobs::status.eq(serde_json::to_string(&JobStatus::Failed)?),
                jobs::error.eq(Some(error)),
                jobs::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    fn row_to_job(row: DbJob) -> Result<Job> {
        Ok(Job {
            id: row.id,
            job_type: row.job_type,
            payload: row.payload,
            status: serde_json::from_str(&row.status)?,
            result: row.result,
            error: row.error,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
