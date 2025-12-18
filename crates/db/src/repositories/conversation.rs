use common::models::{ChatMessage, Conversation};
use common::{Error, Result};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

pub struct ConversationRepository {
    pool: PgPool,
}

#[derive(Debug, FromRow)]
struct Row {
    id: Uuid,
    messages: serde_json::Value,
    agent_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl Row {
    fn into_conversation(self) -> Result<Conversation> {
        Ok(Conversation {
            id: self.id,
            messages: serde_json::from_value(self.messages)?,
            agent_id: self.agent_id,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

impl ConversationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, conv: &Conversation) -> Result<Conversation> {
        sqlx::query_as::<_, Row>(
            "INSERT INTO conversations (id, messages, agent_id, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, messages, agent_id, created_at, updated_at",
        )
        .bind(conv.id)
        .bind(serde_json::to_value(&conv.messages)?)
        .bind(&conv.agent_id)
        .bind(conv.created_at)
        .bind(conv.updated_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
        .into_conversation()
    }

    pub async fn get(&self, id: &Uuid) -> Result<Option<Conversation>> {
        match sqlx::query_as::<_, Row>(
            "SELECT id, messages, agent_id, created_at, updated_at FROM conversations WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?
        {
            Some(row) => Ok(Some(row.into_conversation()?)),
            None => Ok(None),
        }
    }

    pub async fn get_or_create(&self, id: Option<&Uuid>) -> Result<Conversation> {
        if let Some(id) = id {
            if let Some(conv) = self.get(id).await? {
                return Ok(conv);
            }
        }
        self.create(&Conversation::new()).await
    }

    pub async fn update_messages(&self, id: &Uuid, messages: &[ChatMessage]) -> Result<()> {
        sqlx::query("UPDATE conversations SET messages = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(serde_json::to_value(messages)?)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub async fn add_message(&self, id: &Uuid, message: ChatMessage) -> Result<()> {
        let mut messages = self
            .get(id)
            .await?
            .ok_or_else(|| Error::NotFound("conversation not found".into()))?
            .messages;
        messages.push(message);
        self.update_messages(id, &messages).await
    }
}
