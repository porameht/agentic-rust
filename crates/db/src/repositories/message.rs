//! Message repository using Diesel ORM.
//! Reactions are stored directly in the messages table.

use crate::models::{Message, MessageReactionUpdate, NewMessage};
use crate::pool::DbPool;
use crate::schema::messages;
use chrono::Utc;
use common::{Error, Result};
use diesel::prelude::*;
use uuid::Uuid;

pub struct MessageRepository {
    pool: DbPool,
}

impl MessageRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get(&self, id: &Uuid) -> Result<Option<Message>> {
        let mut conn = self.pool.conn()?;
        messages::table
            .find(id)
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn get_by_conversation(&self, conversation_id: &Uuid) -> Result<Vec<Message>> {
        let mut conn = self.pool.conn()?;
        messages::table
            .filter(messages::conversation_id.eq(conversation_id))
            .order(messages::created_at.asc())
            .load(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn create(&self, conversation_id: Uuid, role: &str, content: &str) -> Result<Message> {
        let mut conn = self.pool.conn()?;
        let new_msg = NewMessage {
            id: Uuid::new_v4(),
            conversation_id,
            role,
            content,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        diesel::insert_into(messages::table)
            .values(&new_msg)
            .returning(Message::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn delete(&self, id: &Uuid) -> Result<bool> {
        let mut conn = self.pool.conn()?;
        let count = diesel::delete(messages::table.find(id))
            .execute(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;
        Ok(count > 0)
    }

    /// Add or update a reaction on a message
    pub fn set_reaction(
        &self,
        message_id: &Uuid,
        reaction_type: &str,
        user_id: Option<&str>,
        feedback: Option<&str>,
    ) -> Result<Message> {
        let mut conn = self.pool.conn()?;

        let update = MessageReactionUpdate {
            reaction_type: Some(reaction_type),
            reaction_user_id: user_id,
            reaction_feedback: feedback,
            reacted_at: Some(Utc::now()),
        };

        diesel::update(messages::table.find(message_id))
            .set(&update)
            .returning(Message::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    /// Remove reaction from a message
    pub fn clear_reaction(&self, message_id: &Uuid) -> Result<Message> {
        let mut conn = self.pool.conn()?;

        diesel::update(messages::table.find(message_id))
            .set((
                messages::reaction_type.eq(None::<String>),
                messages::reaction_user_id.eq(None::<String>),
                messages::reaction_feedback.eq(None::<String>),
                messages::reacted_at.eq(None::<chrono::DateTime<Utc>>),
            ))
            .returning(Message::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    /// Get reaction for a message (if any)
    pub fn get_reaction(&self, message_id: &Uuid) -> Result<Option<(String, Option<String>, Option<String>)>> {
        let msg = self.get(message_id)?;
        match msg {
            Some(m) => Ok(m.reaction_type.map(|rt| (rt, m.reaction_user_id, m.reaction_feedback))),
            None => Ok(None),
        }
    }
}
