//! Document repository for storing and retrieving documents.

use common::models::Document;
use common::{Error, Result};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for document operations
pub struct DocumentRepository {
    pool: PgPool,
}

impl DocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new document
    pub async fn create(&self, document: &Document) -> Result<Document> {
        let row = sqlx::query_as!(
            DocumentRow,
            r#"
            INSERT INTO documents (id, title, content, metadata, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, title, content, metadata, created_at, updated_at
            "#,
            document.id,
            document.title,
            document.content,
            document.metadata,
            document.created_at,
            document.updated_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.into())
    }

    /// Get a document by ID
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<Document>> {
        let row = sqlx::query_as!(
            DocumentRow,
            r#"
            SELECT id, title, content, metadata, created_at, updated_at
            FROM documents
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(row.map(Into::into))
    }

    /// List all documents
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Document>> {
        let rows = sqlx::query_as!(
            DocumentRow,
            r#"
            SELECT id, title, content, metadata, created_at, updated_at
            FROM documents
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Delete a document
    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            DELETE FROM documents
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}

// Internal row type for sqlx
#[derive(Debug)]
struct DocumentRow {
    id: Uuid,
    title: String,
    content: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<DocumentRow> for Document {
    fn from(row: DocumentRow) -> Self {
        Document {
            id: row.id,
            title: row.title,
            content: row.content,
            metadata: row.metadata,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
