//! Message and reaction repository using Diesel ORM.

use crate::models::{Message, MessageReaction, NewMessage, NewMessageReaction};
use crate::pool::DbPool;
use crate::schema::{message_reactions, messages};
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

    pub fn add_reaction(
        &self,
        message_id: Uuid,
        reaction_type: &str,
        user_id: Option<&str>,
        feedback: Option<&str>,
    ) -> Result<MessageReaction> {
        let mut conn = self.pool.conn()?;
        let new_reaction = NewMessageReaction {
            id: Uuid::new_v4(),
            message_id,
            user_id,
            reaction_type,
            feedback,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        diesel::insert_into(message_reactions::table)
            .values(&new_reaction)
            .returning(MessageReaction::as_returning())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn get_reaction(&self, message_id: &Uuid, user_id: Option<&str>) -> Result<Option<MessageReaction>> {
        let mut conn = self.pool.conn()?;
        let mut query = message_reactions::table
            .filter(message_reactions::message_id.eq(message_id))
            .into_boxed();

        if let Some(uid) = user_id {
            query = query.filter(message_reactions::user_id.eq(uid));
        }

        query
            .first(&mut conn)
            .optional()
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn get_reactions(&self, message_id: &Uuid) -> Result<Vec<MessageReaction>> {
        let mut conn = self.pool.conn()?;
        message_reactions::table
            .filter(message_reactions::message_id.eq(message_id))
            .order(message_reactions::created_at.desc())
            .load(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))
    }

    pub fn count_reactions(&self, message_id: &Uuid) -> Result<(i64, i64)> {
        let mut conn = self.pool.conn()?;

        let likes: i64 = message_reactions::table
            .filter(message_reactions::message_id.eq(message_id))
            .filter(message_reactions::reaction_type.eq("like"))
            .count()
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        let dislikes: i64 = message_reactions::table
            .filter(message_reactions::message_id.eq(message_id))
            .filter(message_reactions::reaction_type.eq("dislike"))
            .count()
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e.to_string()))?;

        Ok((likes, dislikes))
    }

    pub fn update_reaction(
        &self,
        message_id: &Uuid,
        user_id: &str,
        reaction_type: &str,
        feedback: Option<&str>,
    ) -> Result<MessageReaction> {
        let mut conn = self.pool.conn()?;

        // Try to update existing reaction
        let updated = diesel::update(
            message_reactions::table
                .filter(message_reactions::message_id.eq(message_id))
                .filter(message_reactions::user_id.eq(user_id)),
        )
        .set((
            message_reactions::reaction_type.eq(reaction_type),
            message_reactions::feedback.eq(feedback),
        ))
        .returning(MessageReaction::as_returning())
        .get_result(&mut conn)
        .optional()
        .map_err(|e| Error::Database(e.to_string()))?;

        match updated {
            Some(r) => Ok(r),
            None => self.add_reaction(*message_id, reaction_type, Some(user_id), feedback),
        }
    }

    pub fn delete_reaction(&self, message_id: &Uuid, user_id: &str) -> Result<bool> {
        let mut conn = self.pool.conn()?;
        let count = diesel::delete(
            message_reactions::table
                .filter(message_reactions::message_id.eq(message_id))
                .filter(message_reactions::user_id.eq(user_id)),
        )
        .execute(&mut conn)
        .map_err(|e| Error::Database(e.to_string()))?;
        Ok(count > 0)
    }
}
