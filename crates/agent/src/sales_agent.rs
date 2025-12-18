//! Sales Agent implementation for customer support and product recommendations.
//!
//! This agent is designed to:
//! - Answer questions about the company
//! - Recommend products based on customer needs
//! - Provide brochures and documents for download
//! - Handle FAQ and policies
//!
//! Prompts are loaded from `common::global_config()` singleton.

use crate::tools::{self, Tool};
use common::constants::{DEFAULT_MODEL, DEFAULT_TEMPERATURE, DEFAULT_TOP_K, SALES_AGENT_TOOLS};
use common::global_config;
use common::models::AgentConfig;

/// Create a sales agent configuration from config file
pub fn create_sales_agent_config(language: &str) -> AgentConfig {
    let config = global_config();

    // Get sales agent config
    if let Some(agent_config) = config.get_agent("sales") {
        let lang_key = match language {
            "th" | "thai" => "th",
            _ => "en",
        };

        let preamble = agent_config
            .prompts
            .get(lang_key)
            .or_else(|| agent_config.prompts.get("default"))
            .map(|p| p.prompt.clone())
            .unwrap_or_else(|| "You are a helpful sales assistant.".to_string());

        AgentConfig {
            id: agent_config.id.clone(),
            name: agent_config.name.clone(),
            description: agent_config.description.clone().unwrap_or_default(),
            model: agent_config.default_model.clone(),
            preamble,
            temperature: agent_config.temperature,
            top_k_documents: agent_config.top_k_documents,
            tools: agent_config.tools.clone(),
        }
    } else {
        // Fallback to defaults using constants
        AgentConfig {
            id: "sales-agent".to_string(),
            name: "Sales Agent".to_string(),
            description: "AI assistant for sales support and product recommendations".to_string(),
            model: DEFAULT_MODEL.to_string(),
            preamble: "You are a helpful sales assistant.".to_string(),
            temperature: DEFAULT_TEMPERATURE,
            top_k_documents: DEFAULT_TOP_K,
            tools: SALES_AGENT_TOOLS.iter().map(|s| s.to_string()).collect(),
        }
    }
}

/// Sales agent builder with all tools pre-configured
pub struct SalesAgentBuilder {
    config: AgentConfig,
    tools: Vec<Box<dyn Tool>>,
    language: String,
}

impl SalesAgentBuilder {
    /// Create a new sales agent builder with Thai language (default)
    pub fn new() -> Self {
        Self {
            config: create_sales_agent_config("th"),
            tools: tools::create_sales_agent_tools(),
            language: "th".to_string(),
        }
    }

    /// Set language for the agent
    pub fn language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self.config = create_sales_agent_config(language);
        self
    }

    /// Set custom model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature;
        self
    }

    /// Set number of documents to retrieve for RAG
    pub fn top_k_documents(mut self, top_k: usize) -> Self {
        self.config.top_k_documents = top_k;
        self
    }

    /// Override preamble with custom prompt
    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.config.preamble = preamble.into();
        self
    }

    /// Add custom context (appends to existing preamble)
    pub fn with_custom_context(mut self, context: &str) -> Self {
        self.config.preamble = format!(
            "{}\n\n## Additional Context:\n{}",
            self.config.preamble, context
        );
        self
    }

    /// Build the agent configuration
    pub fn build(self) -> (AgentConfig, Vec<Box<dyn Tool>>) {
        (self.config, self.tools)
    }
}

impl Default for SalesAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sales_agent_builder() {
        let (config, tools) = SalesAgentBuilder::new()
            .language("th")
            .model("gpt-4-turbo")
            .temperature(0.8)
            .build();

        assert_eq!(config.id, "sales-agent");
        assert_eq!(config.model, "gpt-4-turbo");
        assert_eq!(config.temperature, 0.8);
        assert_eq!(tools.len(), 4);
    }

    #[test]
    fn test_sales_agent_english() {
        let config = create_sales_agent_config("en");
        assert!(config.preamble.contains("sales") || config.preamble.contains("Sales"));
    }

    #[test]
    fn test_custom_preamble() {
        let (config, _) = SalesAgentBuilder::new().preamble("Custom preamble").build();

        assert_eq!(config.preamble, "Custom preamble");
    }
}
