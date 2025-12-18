//! Agent builder pattern for constructing LLM agents.

use common::models::AgentConfig;

/// Builder for creating LLM agents
pub struct AgentBuilder {
    config: AgentConfig,
}

impl AgentBuilder {
    /// Create a new agent builder with a model
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            config: AgentConfig {
                model: model.into(),
                ..Default::default()
            },
        }
    }

    /// Set the agent ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.config.id = id.into();
        self
    }

    /// Set the agent name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Set the agent description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.description = description.into();
        self
    }

    /// Set the system preamble/prompt
    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.config.preamble = preamble.into();
        self
    }

    /// Set the temperature for generation
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature;
        self
    }

    /// Set the number of documents to retrieve for RAG
    pub fn top_k_documents(mut self, top_k: usize) -> Self {
        self.config.top_k_documents = top_k;
        self
    }

    /// Add a tool to the agent
    pub fn tool(mut self, tool_name: impl Into<String>) -> Self {
        self.config.tools.push(tool_name.into());
        self
    }

    /// Build the agent configuration
    pub fn build(self) -> AgentConfig {
        self.config
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new("gpt-4")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        let config = AgentBuilder::new("gpt-4")
            .id("test-agent")
            .name("Test Agent")
            .preamble("You are a helpful assistant.")
            .temperature(0.5)
            .top_k_documents(3)
            .tool("search")
            .build();

        assert_eq!(config.id, "test-agent");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.top_k_documents, 3);
        assert!(config.tools.contains(&"search".to_string()));
    }
}
