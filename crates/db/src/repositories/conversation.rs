//! Conversation repository using Diesel ORM.

use crate::models::{Conversation as DbConversation, Message as DbMessage, NewConversation, NewMessage};
use crate::pool::DbPool;
use crate::schema::{conversations, messages};
use chrono::Utc;
use common::models::{ChatMessage, Conversation, MessageRole};
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
        let now = Utc::now();
        let new_conv = NewConversation {
            id: conv.id,
            agent_id: conv.agent_id.clone(),
            created_at: now,
            updated_at: now,
        };

        let row: DbConversation = diesel::insert_into(conversations::table)
            .values(&new_conv)
            .returning(DbConversation::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        // Insert messages if any
        for msg in &conv.messages {
            let new_msg = NewMessage {
                id: Uuid::new_v4(),
                conversation_id: row.id,
                role: match msg.role {
                    MessageRole::User => "user",
                    MessageRole::Assistant => "assistant",
                    MessageRole::System => "system",
                },
                content: &msg.content,
                metadata: serde_json::json!({}),
                created_at: Utc::now(),
            };
            diesel::insert_into(messages::table)
                .values(&new_msg)
                .execute(&mut conn)
                .map_err(|e| Error::Database(e.to_string()))?;
        }

        self.get_with_messages(&row.id)?.ok_or_else(|| Error::Database("failed to create".into()))
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<DbConversation>> {
        let mut conn = self.pool.conn()?;
        conversations::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn get_with_messages(&self, id: &Uuid) -> Result<Option<Conversation>> {
        let mut conn = self.pool.conn()?;

        let conv: Option<DbConversation> = conversations::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))?;

        match conv {
            Some(c) => {
                let msgs: Vec<DbMessage> = messages::table
                    .filter(messages::conversation_id.eq(id))
                    .order(messages::created_at.asc())
                    .load(&mut conn)
                    .map_err(|e| Error::Database(e.to_string()))?;

                let chat_messages: Vec<ChatMessage> = msgs
                    .into_iter()
                    .map(|m| ChatMessage {
                        role: match m.role.as_str() {
                            "assistant" => MessageRole::Assistant,
                            "system" => MessageRole::System,
                            _ => MessageRole::User,
                        },
                        content: m.content,
                    })
                    .collect();

                Ok(Some(Conversation {
                    id: c.id,
                    messages: chat_messages,
                    agent_id: c.agent_id,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn get_or_create(&self, id: Option<&Uuid>) -> Result<Conversation> {
        if let Some(id) = id {
            if let Some(conv) = self.get_with_messages(id)? {
                return Ok(conv);
            }
        }
        self.create(&Conversation::new())
    }

    pub fn add_message(&self, conversation_id: &Uuid, message: ChatMessage) -> Result<DbMessage> {
        let mut conn = self.pool.conn()?;

        let new_msg = NewMessage {
            id: Uuid::new_v4(),
            conversation_id: *conversation_id,
            role: match message.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::System => "system",
            },
            content: &message.content,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        let msg: DbMessage = diesel::insert_into(messages::table)
            .values(&new_msg)
            .returning(DbMessage::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        // Update conversation's updated_at
        diesel::update(conversations::table.find(conversation_id))
            .set(conversations::updated_at.eq(Utc::now()))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok(msg)
    }

    pub fn get_messages(&self, conversation_id: &Uuid) -> Result<Vec<DbMessage>> {
        let mut conn = self.pool.conn()?;
        messages::table
            .filter(messages::conversation_id.eq(conversation_id))
            .order(messages::created_at.asc())
            .load(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }
}
