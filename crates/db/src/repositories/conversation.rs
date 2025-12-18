//! Conversation repository using Diesel ORM.

use crate::models::{Conversation as DbConversation, NewConversation};
use crate::pool::DbPool;
use crate::schema::conversations;
use chrono::Utc;
use common::models::{ChatMessage, Conversation};
use common::{Error, Result};
use diesel::prelude::*;
use uuid::Uuid;

pub struct ConversationRepository {
    pool: DbPool,
}

impl ConversationRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, conv: &Conversation) -> Result<Conversation> {
        let mut conn = self.pool.conn()?;
        let new_conv = NewConversation {
            id: conv.id,
            messages: serde_json::to_value(&conv.messages)?,
            agent_id: conv.agent_id.clone(),
            created_at: conv.created_at,
            updated_at: conv.updated_at,
        };

        let row: DbConversation = diesel::insert_into(conversations::table)
            .values(&new_conv)
            .returning(DbConversation::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        Self::row_to_conversation(row)
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Conversation>> {
        let mut conn = self.pool.conn()?;
        let row: Option<DbConversation> = conversations::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(Self::row_to_conversation(r)?)),
            None => Ok(None),
        }
    }

    pub fn get_or_create(&self, id: Option<&Uuid>) -> Result<Conversation> {
        if let Some(id) = id {
            if let Some(conv) = self.get(id)? {
                return Ok(conv);
            }
        }
        self.create(&Conversation::new())
    }

    pub fn update_messages(&self, id: &Uuid, messages: &[ChatMessage]) -> Result<()> {
        let mut conn = self.pool.conn()?;
        diesel::update(conversations::table.find(id))
            .set((
                conversations::messages.eq(serde_json::to_value(messages)?),
                conversations::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(())
    }

    pub fn add_message(&self, id: &Uuid, message: ChatMessage) -> Result<()> {
        let mut messages = self
            .get(id)?
            .ok_or_else(|| Error::NotFound("conversation not found".into()))?
            .messages;
        messages.push(message);
        self.update_messages(id, &messages)
    }

    fn row_to_conversation(row: DbConversation) -> Result<Conversation> {
        Ok(Conversation {
            id: row.id,
            messages: serde_json::from_value(row.messages)?,
            agent_id: row.agent_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
