//! Conversation repository for storing chat history.

use common::models::{ChatMessage, Conversation};
use common::{Error, Result};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Repository for conversation operations
pub struct ConversationRepository {
    pool: PgPool,
}

// Internal row type for sqlx
#[derive(Debug, FromRow)]
struct ConversationRow {
    id: Uuid,
    messages: serde_json::Value,
    agent_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl ConversationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new conversation
    pub async fn create(&self, conversation: &Conversation) -> Result<Conversation> {
        let messages_json = serde_json::to_value(&conversation.messages)?;

        let row: ConversationRow = sqlx::query_as(
            r#"
            INSERT INTO conversations (id, messages, agent_id, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, messages, agent_id, created_at, updated_at
            "#,
        )
        .bind(conversation.id)
        .bind(&messages_json)
        .bind(&conversation.agent_id)
        .bind(conversation.created_at)
        .bind(conversation.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let messages: Vec<ChatMessage> = serde_json::from_value(row.messages)?;

        Ok(Conversation {
            id: row.id,
            messages,
            agent_id: row.agent_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }

    /// Get a conversation by ID
    pub async fn get_by_id(&self, id: &Uuid) -> Result<Option<Conversation>> {
        let row: Option<ConversationRow> = sqlx::query_as(
            r#"
            SELECT id, messages, agent_id, created_at, updated_at
            FROM conversations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let messages: Vec<ChatMessage> = serde_json::from_value(r.messages)?;
                Ok(Some(Conversation {
                    id: r.id,
                    messages,
                    agent_id: r.agent_id,
                    created_at: r.created_at,
                    updated_at: r.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    /// Get or create a conversation
    pub async fn get_or_create(&self, id: Option<&Uuid>) -> Result<Conversation> {
        if let Some(conversation_id) = id {
            if let Some(conv) = self.get_by_id(conversation_id).await? {
                return Ok(conv);
            }
        }

        let conversation = Conversation::new();
        self.create(&conversation).await
    }

    /// Update conversation messages
    pub async fn update_messages(&self, id: &Uuid, messages: &[ChatMessage]) -> Result<()> {
        let messages_json = serde_json::to_value(messages)?;

        sqlx::query(
            r#"
            UPDATE conversations
            SET messages = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&messages_json)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Add a message to a conversation
    pub async fn add_message(&self, id: &Uuid, message: ChatMessage) -> Result<()> {
        // Get current messages
        let conversation = self
            .get_by_id(id)
            .await?
            .ok_or_else(|| Error::NotFound("Conversation not found".to_string()))?;

        let mut messages = conversation.messages;
        messages.push(message);

        self.update_messages(id, &messages).await
    }
}
