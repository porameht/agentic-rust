//! RAG-enabled agent implementation.

use common::models::{AgentConfig, SearchResult};
use common::Result;
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
            prompt.push('\n');
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
    use common::models::DocumentChunk;
    use uuid::Uuid;

    fn create_test_chunk(content: &str, index: usize) -> DocumentChunk {
        DocumentChunk {
            id: Uuid::new_v4(),
            document_id: Uuid::new_v4(),
            content: content.to_string(),
            chunk_index: index,
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn test_chat_response_structure() {
        let response = ChatResponse {
            message: "Test message".to_string(),
            sources: vec![],
        };

        assert_eq!(response.message, "Test message");
        assert!(response.sources.is_empty());
    }

    #[test]
    fn test_chat_response_with_sources() {
        let sources = vec![
            SearchResult {
                chunk: create_test_chunk("Content 1", 0),
                score: 0.95,
            },
            SearchResult {
                chunk: create_test_chunk("Content 2", 1),
                score: 0.85,
            },
        ];

        let response = ChatResponse {
            message: "Answer based on sources".to_string(),
            sources,
        };

        assert_eq!(response.sources.len(), 2);
        assert_eq!(response.sources[0].score, 0.95);
        assert_eq!(response.sources[1].score, 0.85);
    }

    #[test]
    fn test_chat_response_clone() {
        let response = ChatResponse {
            message: "Original".to_string(),
            sources: vec![],
        };

        let cloned = response.clone();
        assert_eq!(cloned.message, "Original");
    }

    #[test]
    fn test_chat_response_debug() {
        let response = ChatResponse {
            message: "Debug test".to_string(),
            sources: vec![],
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_search_result_score_ordering() {
        let mut results = vec![
            SearchResult {
                chunk: create_test_chunk("Low score", 0),
                score: 0.5,
            },
            SearchResult {
                chunk: create_test_chunk("High score", 1),
                score: 0.9,
            },
        ];

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        assert_eq!(results[0].score, 0.9);
        assert_eq!(results[1].score, 0.5);
    }

    #[test]
    fn test_chat_response_with_multiple_sources() {
        let sources: Vec<SearchResult> = (0..5)
            .map(|i| SearchResult {
                chunk: create_test_chunk(&format!("Content {}", i), i),
                score: 1.0 - (i as f32 * 0.1),
            })
            .collect();

        let response = ChatResponse {
            message: "Multiple sources".to_string(),
            sources,
        };

        assert_eq!(response.sources.len(), 5);
        assert_eq!(response.sources[0].score, 1.0);
        assert_eq!(response.sources[4].score, 0.6);
    }

    #[test]
    fn test_document_chunk_content() {
        let chunk = create_test_chunk("Test content", 0);
        assert_eq!(chunk.content, "Test content");
        assert_eq!(chunk.chunk_index, 0);
    }
}
