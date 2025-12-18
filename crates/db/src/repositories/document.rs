//! Document repository using Diesel ORM.

use crate::models::{Document, NewDocument};
use crate::pool::DbPool;
use crate::schema::documents;
use chrono::Utc;
use common::{Error, Result};
use diesel::prelude::*;
use uuid::Uuid;

pub struct DocumentRepository {
    pool: DbPool,
}

impl DocumentRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, doc: &common::models::Document) -> Result<Document> {
        let mut conn = self.pool.conn()?;
        let new_doc = NewDocument {
            id: doc.id,
            title: &doc.title,
            content: &doc.content,
            metadata: doc.metadata.clone(),
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        };

        diesel::insert_into(documents::table)
            .values(&new_doc)
            .returning(Document::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Document>> {
        let mut conn = self.pool.conn()?;
        documents::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn list(&self, limit: i64, offset: i64) -> Result<Vec<Document>> {
        let mut conn = self.pool.conn()?;
        documents::table
            .order(documents::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn update(
        &self,
        id: &Uuid,
        title: &str,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<Document> {
        let mut conn = self.pool.conn()?;
        diesel::update(documents::table.find(id))
            .set((
                documents::title.eq(title),
                documents::content.eq(content),
                documents::metadata.eq(metadata),
                documents::updated_at.eq(Utc::now()),
            ))
            .returning(Document::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn delete(&self, id: &Uuid) -> Result<bool> {
        let mut conn = self.pool.conn()?;
        let count = diesel::delete(documents::table.find(id))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(count > 0)
    }
}
