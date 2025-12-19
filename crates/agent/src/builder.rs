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

    #[test]
    fn test_builder_default() {
        let builder = AgentBuilder::default();
        let config = builder.build();

        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn test_builder_multiple_tools() {
        let config = AgentBuilder::new("gpt-4")
            .tool("search")
            .tool("calculator")
            .tool("weather")
            .build();

        assert_eq!(config.tools.len(), 3);
        assert!(config.tools.contains(&"search".to_string()));
        assert!(config.tools.contains(&"calculator".to_string()));
        assert!(config.tools.contains(&"weather".to_string()));
    }

    #[test]
    fn test_builder_description() {
        let config = AgentBuilder::new("gpt-4")
            .description("A test agent for unit testing")
            .build();

        assert_eq!(config.description, "A test agent for unit testing");
    }

    #[test]
    fn test_builder_different_models() {
        let models = ["gpt-4", "gpt-3.5-turbo", "claude-3-opus", "llama-3"];

        for model in models {
            let config = AgentBuilder::new(model).build();
            assert_eq!(config.model, model);
        }
    }

    #[test]
    fn test_builder_temperature_bounds() {
        // Test low temperature
        let config_low = AgentBuilder::new("gpt-4").temperature(0.0).build();
        assert_eq!(config_low.temperature, 0.0);

        // Test high temperature
        let config_high = AgentBuilder::new("gpt-4").temperature(2.0).build();
        assert_eq!(config_high.temperature, 2.0);
    }

    #[test]
    fn test_builder_top_k_variations() {
        for k in [1, 5, 10, 20, 100] {
            let config = AgentBuilder::new("gpt-4").top_k_documents(k).build();
            assert_eq!(config.top_k_documents, k);
        }
    }

    #[test]
    fn test_builder_chaining() {
        let config = AgentBuilder::new("gpt-4")
            .id("agent-1")
            .name("Agent One")
            .description("First agent")
            .preamble("System prompt")
            .temperature(0.7)
            .top_k_documents(5)
            .tool("tool1")
            .tool("tool2")
            .build();

        assert_eq!(config.id, "agent-1");
        assert_eq!(config.name, "Agent One");
        assert_eq!(config.description, "First agent");
        assert_eq!(config.preamble, "System prompt");
        assert_eq!(config.temperature, 0.7);
        assert_eq!(config.top_k_documents, 5);
        assert_eq!(config.tools.len(), 2);
    }

    #[test]
    fn test_builder_empty_preamble() {
        let config = AgentBuilder::new("gpt-4").preamble("").build();
        assert_eq!(config.preamble, "");
    }

    #[test]
    fn test_builder_unicode_support() {
        let config = AgentBuilder::new("gpt-4")
            .name("เอเจนต์ภาษาไทย")
            .preamble("คุณคือผู้ช่วยที่เป็นประโยชน์")
            .build();

        assert_eq!(config.name, "เอเจนต์ภาษาไทย");
        assert!(config.preamble.contains("คุณคือ"));
    }
}
