//! Prompt templates for agents.
//!
//! Prompts are loaded from `common::global_config()` singleton.
//! Configure via config/prompts.toml or use programmatic defaults.

use common::global_config;
use common::prompt_config::PromptConfig;

/// Get the global prompt configuration
///
/// Delegates to `common::global_config()` - the single source of truth.
pub fn get_config() -> &'static PromptConfig {
    global_config()
}

/// Get a template prompt by name
pub fn get_template(name: &str) -> Option<String> {
    get_config().get_template(name).map(|t| t.prompt.clone())
}

/// Get agent prompt for a specific language
pub fn get_agent_prompt(agent: &str, language: &str) -> Option<String> {
    get_config()
        .get_agent_prompt(agent, language)
        .map(|s| s.to_string())
}

/// Template access module
///
/// Templates are now managed by `PromptConfig` in the common crate.
/// This module provides backward-compatible access.
pub mod templates {
    use super::get_config;

    /// Get template by name from config
    pub fn get(name: &str) -> Option<String> {
        get_config().get_template(name).map(|t| t.prompt.clone())
    }
}

/// Prompt builder for constructing custom prompts
pub struct PromptBuilder {
    parts: Vec<String>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Start with a template from config
    pub fn from_template(template_name: &str) -> Self {
        let mut builder = Self::new();
        if let Some(prompt) = get_template(template_name) {
            builder.parts.push(prompt);
        }
        builder
    }

    /// Add a system instruction
    pub fn system(mut self, instruction: impl Into<String>) -> Self {
        self.parts.push(format!("System: {}", instruction.into()));
        self
    }

    /// Add raw content without prefix
    pub fn raw(mut self, content: impl Into<String>) -> Self {
        self.parts.push(content.into());
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

    #[test]
    fn test_from_template() {
        let prompt = PromptBuilder::from_template("general_assistant")
            .query("Hello")
            .build();

        assert!(prompt.contains("helpful") || prompt.contains("assistant"));
    }

    #[test]
    fn test_templates_get() {
        // Templates are now loaded from config
        let template = templates::get("general_assistant");
        assert!(template.is_some());
        assert!(template.unwrap().contains("helpful"));
        assert!(templates::get("nonexistent").is_none());
    }
}
