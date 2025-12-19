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

    #[test]
    fn test_prompt_builder_default() {
        let builder = PromptBuilder::default();
        let prompt = builder.build();
        assert!(prompt.is_empty());
    }

    #[test]
    fn test_prompt_builder_raw() {
        let prompt = PromptBuilder::new()
            .raw("Raw content without prefix")
            .build();

        assert_eq!(prompt, "Raw content without prefix");
    }

    #[test]
    fn test_prompt_builder_empty_context() {
        let prompt = PromptBuilder::new()
            .system("System instruction")
            .context(&[])
            .query("Question")
            .build();

        assert!(prompt.contains("System instruction"));
        assert!(prompt.contains("Question"));
        assert!(!prompt.contains("Context:"));
    }

    #[test]
    fn test_prompt_builder_multiple_systems() {
        let prompt = PromptBuilder::new()
            .system("First instruction")
            .system("Second instruction")
            .build();

        assert!(prompt.contains("First instruction"));
        assert!(prompt.contains("Second instruction"));
    }

    #[test]
    fn test_prompt_builder_context_numbering() {
        let prompt = PromptBuilder::new()
            .context(&["Doc A", "Doc B", "Doc C"])
            .build();

        assert!(prompt.contains("[1]: Doc A"));
        assert!(prompt.contains("[2]: Doc B"));
        assert!(prompt.contains("[3]: Doc C"));
    }

    #[test]
    fn test_prompt_builder_unicode() {
        let prompt = PromptBuilder::new()
            .system("คุณเป็นผู้ช่วย")
            .context(&["เอกสาร 1", "เอกสาร 2"])
            .query("คำถามภาษาไทย")
            .build();

        assert!(prompt.contains("คุณเป็นผู้ช่วย"));
        assert!(prompt.contains("เอกสาร 1"));
        assert!(prompt.contains("คำถามภาษาไทย"));
    }

    #[test]
    fn test_prompt_builder_chaining() {
        let prompt = PromptBuilder::new()
            .system("Instruction 1")
            .raw("Raw section")
            .context(&["Context doc"])
            .query("Final question")
            .build();

        let parts: Vec<&str> = prompt.split("\n\n").collect();
        assert!(parts.len() >= 4);
    }

    #[test]
    fn test_get_config() {
        let config = get_config();
        // Verify config is accessible
        assert!(config.get_template("general_assistant").is_some());
    }

    #[test]
    fn test_get_template_function() {
        let template = get_template("general_assistant");
        assert!(template.is_some());

        let nonexistent = get_template("this_does_not_exist");
        assert!(nonexistent.is_none());
    }

    #[test]
    fn test_prompt_builder_long_context() {
        let long_docs: Vec<&str> = (0..10)
            .map(|i| match i {
                0 => "Document zero",
                1 => "Document one",
                2 => "Document two",
                3 => "Document three",
                4 => "Document four",
                5 => "Document five",
                6 => "Document six",
                7 => "Document seven",
                8 => "Document eight",
                _ => "Document nine",
            })
            .collect();

        let prompt = PromptBuilder::new().context(&long_docs).build();

        assert!(prompt.contains("[1]:"));
        assert!(prompt.contains("[10]:"));
    }
}
