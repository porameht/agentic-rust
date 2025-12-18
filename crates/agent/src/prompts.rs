//! Prompt templates for agents.

/// Default system prompts for different agent types
pub mod templates {
    /// General-purpose assistant prompt
    pub const GENERAL_ASSISTANT: &str = r#"You are a helpful AI assistant. You provide accurate, helpful, and concise responses to user questions. When you don't know something, you say so honestly."#;

    /// RAG-enabled assistant prompt
    pub const RAG_ASSISTANT: &str = r#"You are a helpful AI assistant with access to a knowledge base. When answering questions:
1. Use the provided context to inform your answers
2. If the context doesn't contain relevant information, say so
3. Cite your sources when possible
4. Be accurate and concise"#;

    /// Code assistant prompt
    pub const CODE_ASSISTANT: &str = r#"You are an expert software engineer and coding assistant. You help users with:
- Writing clean, efficient code
- Debugging issues
- Explaining complex concepts
- Suggesting best practices

Always provide working code examples when appropriate."#;

    /// Document Q&A prompt
    pub const DOCUMENT_QA: &str = r#"You are a document analysis assistant. Your job is to answer questions based on the provided documents. Guidelines:
1. Only use information from the provided context
2. Quote relevant passages when appropriate
3. If the answer isn't in the documents, clearly state that
4. Summarize complex information clearly"#;
}

/// Prompt builder for constructing custom prompts
pub struct PromptBuilder {
    parts: Vec<String>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Add a system instruction
    pub fn system(mut self, instruction: impl Into<String>) -> Self {
        self.parts.push(format!("System: {}", instruction.into()));
        self
    }

    /// Add context documents
    pub fn context(mut self, documents: &[&str]) -> Self {
        if !documents.is_empty() {
            self.parts.push("Context:".to_string());
            for (i, doc) in documents.iter().enumerate() {
                self.parts.push(format!("[{}]: {}", i + 1, doc));
            }
        }
        self
    }

    /// Add the user query
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.parts.push(format!("Question: {}", query.into()));
        self
    }

    /// Build the final prompt
    pub fn build(self) -> String {
        self.parts.join("\n\n")
    }
}

impl Default for PromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new()
            .system("You are a helpful assistant")
            .context(&["Document 1 content", "Document 2 content"])
            .query("What is the answer?")
            .build();

        assert!(prompt.contains("You are a helpful assistant"));
        assert!(prompt.contains("Document 1 content"));
        assert!(prompt.contains("What is the answer?"));
    }
}
