//! RAG-enabled agent implementation.

use common::models::{AgentConfig, ChatMessage, MessageRole, SearchResult};
use common::{Error, Result};
use rag_core::embeddings::EmbeddingModel;
use rag_core::vector_store::VectorStore;
use rag_core::Retriever;

/// A RAG-enabled agent that retrieves context before generating responses
pub struct RagAgent<E: EmbeddingModel, V: VectorStore> {
    config: AgentConfig,
    retriever: Retriever<E, V>,
}

impl<E: EmbeddingModel, V: VectorStore> RagAgent<E, V> {
    /// Create a new RAG agent
    pub fn new(config: AgentConfig, retriever: Retriever<E, V>) -> Self {
        Self { config, retriever }
    }

    /// Process a chat message with RAG context
    pub async fn chat(&self, message: &str) -> Result<ChatResponse> {
        // 1. Retrieve relevant documents
        let context = self.retriever.retrieve(message).await?;

        // 2. Build prompt with context
        let augmented_prompt = self.build_augmented_prompt(message, &context);

        // 3. Generate response (placeholder - integrate with rig here)
        // In a full implementation, this would use rig's completion API
        let response = self.generate_response(&augmented_prompt).await?;

        Ok(ChatResponse {
            message: response,
            sources: context,
        })
    }

    /// Build an augmented prompt with retrieved context
    fn build_augmented_prompt(&self, query: &str, context: &[SearchResult]) -> String {
        let mut prompt = String::new();

        // Add system preamble
        prompt.push_str(&self.config.preamble);
        prompt.push_str("\n\n");

        // Add retrieved context
        if !context.is_empty() {
            prompt.push_str("Use the following context to help answer the question:\n\n");
            for (i, result) in context.iter().enumerate() {
                prompt.push_str(&format!(
                    "[{}] (score: {:.2}): {}\n\n",
                    i + 1,
                    result.score,
                    result.chunk.content
                ));
            }
            prompt.push_str("\n");
        }

        // Add the user query
        prompt.push_str(&format!("Question: {}\n\nAnswer:", query));

        prompt
    }

    /// Generate a response using the LLM
    /// This is a placeholder - in production, use rig's completion API
    async fn generate_response(&self, _prompt: &str) -> Result<String> {
        // TODO: Integrate with rig's completion API
        // Example with rig:
        // let openai = openai::Client::from_env();
        // let agent = openai.agent(&self.config.model)
        //     .preamble(&self.config.preamble)
        //     .temperature(self.config.temperature)
        //     .build();
        // agent.prompt(prompt).await

        Ok("This is a placeholder response. Integrate with rig's completion API for actual LLM responses.".to_string())
    }

    /// Get the agent configuration
    pub fn config(&self) -> &AgentConfig {
        &self.config
    }
}

/// Response from the RAG agent
#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub message: String,
    pub sources: Vec<SearchResult>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rag_core::vector_store::InMemoryVectorStore;

    // Tests would go here with mock embedding model
}
