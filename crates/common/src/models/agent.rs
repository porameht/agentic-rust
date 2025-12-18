//! Agent configuration models.

use serde::{Deserialize, Serialize};

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model: String,
    pub preamble: String,
    pub temperature: f32,
    pub top_k_documents: usize,
    pub tools: Vec<String>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// Conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: uuid::Uuid,
    pub messages: Vec<ChatMessage>,
    pub agent_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Agent".to_string(),
            description: "A helpful AI assistant".to_string(),
            model: "gpt-4".to_string(),
            preamble: "You are a helpful AI assistant.".to_string(),
            temperature: 0.7,
            top_k_documents: 5,
            tools: Vec::new(),
        }
    }
}

impl Conversation {
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            messages: Vec::new(),
            agent_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_message(&mut self, role: MessageRole, content: impl Into<String>) {
        self.messages.push(ChatMessage {
            role,
            content: content.into(),
        });
        self.updated_at = chrono::Utc::now();
    }
}

impl Default for Conversation {
    fn default() -> Self {
        Self::new()
    }
}
