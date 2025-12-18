//! Process chat job for async chat processing.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Job to process a chat message asynchronously
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessChatJob {
    pub job_id: Uuid,
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub agent_id: Option<String>,
}

impl ProcessChatJob {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            job_id: Uuid::new_v4(),
            message: message.into(),
            conversation_id: None,
            agent_id: None,
        }
    }

    pub fn with_conversation(mut self, conversation_id: Uuid) -> Self {
        self.conversation_id = Some(conversation_id);
        self
    }

    pub fn with_agent(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }
}
