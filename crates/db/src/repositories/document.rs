use common::models::Document;
use common::{Error, Result};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct DocumentRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct Row {
    id: Uuid,
    title: String,
    content: String,
    metadata: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<Row> for Document {
    fn from(r: Row) -> Self {
        Self {
            id: r.id,
            title: r.title,
            content: r.content,
            metadata: r.metadata,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

impl DocumentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, doc: &Document) -> Result<Document> {
        sqlx::query_as::<_, Row>(
            "INSERT INTO documents (id, title, content, metadata, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, title, content, metadata, created_at, updated_at",
        )
        .bind(doc.id)
        .bind(&doc.title)
        .bind(&doc.content)
        .bind(&doc.metadata)
        .bind(doc.created_at)
        .bind(doc.updated_at)
        .fetch_one(&self.pool)
        .await
        .map(Into::into)
        .map_err(|e| Error::Database(e.to_string()))
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Document>> {
        sqlx::query_as::<_, Row>(
            "SELECT id, title, content, metadata, created_at, updated_at FROM documents WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map(|r| r.map(Into::into))
        .map_err(|e| Error::Database(e.to_string()))
    }

    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<Document>> {
        sqlx::query_as::<_, Row>(
            "SELECT id, title, content, metadata, created_at, updated_at FROM documents ORDER BY created_at DESC LIMIT $1 OFFSET $2",
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
        .map_err(|e| Error::Database(e.to_string()))
    }

    pub async fn delete(&self, id: &Uuid) -> Result<bool> {
        sqlx::query("DELETE FROM documents WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map(|r| r.rows_affected() > 0)
            .map_err(|e| Error::Database(e.to_string()))
    }
}
