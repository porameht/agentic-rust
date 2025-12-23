//! Integration with rig-core for LLM operations
//!
//! This module provides the bridge between our ReAct agent and rig's LLM providers.
//!
//! # Supported Providers
//!
//! - OpenAI (GPT-4, GPT-3.5)
//! - Anthropic (Claude 3)
//! - Cohere
//! - And more via rig-core
//!
//! # Example
//!
//! ```rust,ignore
//! use agent::rig_integration::{LlmClient, LlmConfig, Provider};
//!
//! let client = LlmClient::new(LlmConfig {
//!     provider: Provider::OpenAI,
//!     model: "gpt-4".to_string(),
//!     temperature: 0.3,
//!     ..Default::default()
//! });
//!
//! let response = client.complete("What is Rust?").await?;
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tracing::debug;

use crate::tools::ToolDefinition;

// ============================================================================
// ERRORS
// ============================================================================

/// Errors from LLM operations
#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Invalid API key for provider: {0}")]
    InvalidApiKey(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("Model not found: {0}")]
    ModelNotFound(String),

    #[error("Context length exceeded: {0}")]
    ContextLengthExceeded(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

// ============================================================================
// PROVIDER CONFIGURATION
// ============================================================================

/// Supported LLM providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    OpenAI,
    Anthropic,
    Cohere,
    Gemini,
    Ollama,
    Custom,
}

impl Default for Provider {
    fn default() -> Self {
        Provider::OpenAI
    }
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::OpenAI => write!(f, "openai"),
            Provider::Anthropic => write!(f, "anthropic"),
            Provider::Cohere => write!(f, "cohere"),
            Provider::Gemini => write!(f, "gemini"),
            Provider::Ollama => write!(f, "ollama"),
            Provider::Custom => write!(f, "custom"),
        }
    }
}

/// Configuration for LLM client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// LLM provider to use
    pub provider: Provider,

    /// Model name (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// API key (optional, can use env vars)
    pub api_key: Option<String>,

    /// Temperature for generation (0.0 - 1.0)
    pub temperature: f32,

    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,

    /// Top-p sampling
    pub top_p: Option<f32>,

    /// Stop sequences
    pub stop_sequences: Vec<String>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Custom base URL (for Ollama or custom endpoints)
    pub base_url: Option<String>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            api_key: None,
            temperature: 0.3,
            max_tokens: Some(2048),
            top_p: None,
            stop_sequences: vec![],
            timeout_secs: 60,
            base_url: None,
        }
    }
}

impl LlmConfig {
    /// Create a new OpenAI configuration
    pub fn openai(model: &str) -> Self {
        Self {
            provider: Provider::OpenAI,
            model: model.to_string(),
            ..Default::default()
        }
    }

    /// Create a new Anthropic configuration
    pub fn anthropic(model: &str) -> Self {
        Self {
            provider: Provider::Anthropic,
            model: model.to_string(),
            ..Default::default()
        }
    }

    /// Create a new Ollama configuration
    pub fn ollama(model: &str, base_url: Option<&str>) -> Self {
        Self {
            provider: Provider::Ollama,
            model: model.to_string(),
            base_url: base_url.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    /// Set temperature
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    /// Set API key
    pub fn with_api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }
}

// ============================================================================
// LLM RESPONSE
// ============================================================================

/// Response from an LLM completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    /// The generated text
    pub content: String,

    /// Model used
    pub model: String,

    /// Token usage
    pub usage: Option<TokenUsage>,

    /// Finish reason
    pub finish_reason: Option<FinishReason>,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Reason for completion finishing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Other(String),
}

// ============================================================================
// LLM CLIENT TRAIT
// ============================================================================

/// Trait for LLM completion clients
#[async_trait]
pub trait CompletionClient: Send + Sync {
    /// Generate a completion for the given prompt
    async fn complete(&self, prompt: &str) -> Result<LlmResponse, LlmError>;

    /// Generate a completion with system message
    async fn complete_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<LlmResponse, LlmError>;

    /// Generate a chat completion with message history
    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse, LlmError>;

    /// Get the model name
    fn model(&self) -> &str;

    /// Get the provider
    fn provider(&self) -> Provider;
}

/// A chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Message role
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

// ============================================================================
// RIG-BASED LLM CLIENT
// ============================================================================

/// LLM client using rig-core
///
/// This client wraps rig's providers to offer a unified interface
/// for our ReAct agent.
pub struct RigLlmClient {
    config: LlmConfig,
    // In production, this would hold the actual rig client
    // For now, we'll implement a mock version
}

impl RigLlmClient {
    /// Create a new client with the given configuration
    pub fn new(config: LlmConfig) -> Self {
        Self { config }
    }

    /// Create from environment variables
    pub fn from_env(provider: Provider, model: &str) -> Self {
        Self::new(LlmConfig {
            provider,
            model: model.to_string(),
            api_key: None, // Will be read from env
            ..Default::default()
        })
    }

    /// Build the prompt with tool information for ReAct
    pub fn build_react_prompt(
        &self,
        query: &str,
        tools: &[ToolDefinition],
        scratchpad: &str,
        context: Option<&str>,
    ) -> String {
        let tool_desc = if tools.is_empty() {
            "You have no tools available.".to_string()
        } else {
            let mut desc = "Available tools:\n".to_string();
            for tool in tools {
                desc.push_str(&format!(
                    "\n- {}: {}\n  Input: {}\n",
                    tool.name,
                    tool.description,
                    serde_json::to_string(&tool.parameters).unwrap_or_default()
                ));
            }
            desc
        };

        let context_str = context
            .map(|c| format!("\nContext:\n{}\n", c))
            .unwrap_or_default();

        format!(
            r#"You are a ReAct agent. Think step by step and use tools when needed.

{tool_desc}
{context_str}
Format your response as:
Thought: <your reasoning>
Action: <tool_name>
Action Input: <json input>

Or if you have the final answer:
Thought: <final reasoning>
Final Answer: <your answer>

Question: {query}
{scratchpad}"#,
            tool_desc = tool_desc,
            context_str = context_str,
            query = query,
            scratchpad = scratchpad
        )
    }
}

#[async_trait]
impl CompletionClient for RigLlmClient {
    async fn complete(&self, prompt: &str) -> Result<LlmResponse, LlmError> {
        // TODO: Implement actual rig integration
        //
        // Example with rig-core:
        // ```rust
        // use rig::providers::openai;
        //
        // let client = openai::Client::from_env();
        // let response = client
        //     .completion(&self.config.model)
        //     .temperature(self.config.temperature)
        //     .max_tokens(self.config.max_tokens.unwrap_or(2048) as i32)
        //     .prompt(prompt)
        //     .await
        //     .map_err(|e| LlmError::ProviderError(e.to_string()))?;
        // ```

        debug!(
            provider = %self.config.provider,
            model = %self.config.model,
            prompt_len = prompt.len(),
            "Calling LLM"
        );

        // For now, return a placeholder response
        // This simulates the ReAct format
        Ok(LlmResponse {
            content: format!(
                "Thought: I received a query. Let me process it.\n\
                 Final Answer: [Placeholder] LLM integration with rig-core is pending. \
                 The agent would process: {}",
                prompt.chars().take(100).collect::<String>()
            ),
            model: self.config.model.clone(),
            usage: Some(TokenUsage {
                prompt_tokens: (prompt.len() / 4) as u32,
                completion_tokens: 50,
                total_tokens: (prompt.len() / 4) as u32 + 50,
            }),
            finish_reason: Some(FinishReason::Stop),
        })
    }

    async fn complete_with_system(
        &self,
        system: &str,
        prompt: &str,
    ) -> Result<LlmResponse, LlmError> {
        let full_prompt = format!("{}\n\n{}", system, prompt);
        self.complete(&full_prompt).await
    }

    async fn chat(&self, messages: Vec<ChatMessage>) -> Result<LlmResponse, LlmError> {
        // Convert messages to a single prompt for now
        let prompt: String = messages
            .iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        self.complete(&prompt).await
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    fn provider(&self) -> Provider {
        self.config.provider
    }
}

// ============================================================================
// TOOL-CALLING CLIENT (FOR AGENTS WITH TOOLS)
// ============================================================================

/// Extended client that supports tool calling
#[async_trait]
pub trait ToolCallingClient: CompletionClient {
    /// Generate a completion with tool definitions
    async fn complete_with_tools(
        &self,
        prompt: &str,
        tools: &[ToolDefinition],
    ) -> Result<ToolCallResponse, LlmError>;
}

/// Response that may contain tool calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResponse {
    /// Text content (if any)
    pub content: Option<String>,

    /// Tool calls requested by the model
    pub tool_calls: Vec<ToolCall>,

    /// Finish reason
    pub finish_reason: Option<FinishReason>,

    /// Token usage
    pub usage: Option<TokenUsage>,
}

/// A tool call requested by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Tool call ID
    pub id: String,

    /// Tool name
    pub name: String,

    /// Arguments as JSON
    pub arguments: serde_json::Value,
}

#[async_trait]
impl ToolCallingClient for RigLlmClient {
    async fn complete_with_tools(
        &self,
        prompt: &str,
        _tools: &[ToolDefinition],
    ) -> Result<ToolCallResponse, LlmError> {
        // TODO: Implement with rig's tool calling support
        //
        // With rig, this would look like:
        // ```rust
        // let agent = client.agent(&self.config.model)
        //     .preamble("You are a helpful assistant")
        //     .tool(Tool1)
        //     .tool(Tool2)
        //     .build();
        //
        // let response = agent.prompt(prompt).await?;
        // ```

        let response = self.complete(prompt).await?;

        Ok(ToolCallResponse {
            content: Some(response.content),
            tool_calls: vec![],
            finish_reason: response.finish_reason,
            usage: response.usage,
        })
    }
}

// ============================================================================
// FACTORY FUNCTIONS
// ============================================================================

/// Create an LLM client for the specified provider
pub fn create_client(config: LlmConfig) -> Arc<dyn CompletionClient> {
    Arc::new(RigLlmClient::new(config))
}

/// Create an OpenAI client
pub fn openai_client(model: &str) -> Arc<dyn CompletionClient> {
    create_client(LlmConfig::openai(model))
}

/// Create an Anthropic client
pub fn anthropic_client(model: &str) -> Arc<dyn CompletionClient> {
    create_client(LlmConfig::anthropic(model))
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert_eq!(config.provider, Provider::OpenAI);
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.temperature, 0.3);
    }

    #[test]
    fn test_llm_config_openai() {
        let config = LlmConfig::openai("gpt-4-turbo")
            .with_temperature(0.5)
            .with_max_tokens(1024);

        assert_eq!(config.provider, Provider::OpenAI);
        assert_eq!(config.model, "gpt-4-turbo");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, Some(1024));
    }

    #[test]
    fn test_llm_config_anthropic() {
        let config = LlmConfig::anthropic("claude-3-opus");
        assert_eq!(config.provider, Provider::Anthropic);
        assert_eq!(config.model, "claude-3-opus");
    }

    #[test]
    fn test_chat_message() {
        let system = ChatMessage::system("You are helpful");
        let user = ChatMessage::user("Hello");
        let assistant = ChatMessage::assistant("Hi there!");

        assert!(matches!(system.role, MessageRole::System));
        assert!(matches!(user.role, MessageRole::User));
        assert!(matches!(assistant.role, MessageRole::Assistant));
    }

    #[tokio::test]
    async fn test_rig_client_complete() {
        let client = RigLlmClient::new(LlmConfig::default());
        let response = client.complete("Hello, world!").await.unwrap();

        assert!(!response.content.is_empty());
        assert_eq!(response.model, "gpt-4");
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(Provider::OpenAI.to_string(), "openai");
        assert_eq!(Provider::Anthropic.to_string(), "anthropic");
        assert_eq!(Provider::Ollama.to_string(), "ollama");
    }

    #[test]
    fn test_build_react_prompt() {
        let client = RigLlmClient::new(LlmConfig::default());
        let tools = vec![ToolDefinition {
            name: "search".to_string(),
            description: "Search the web".to_string(),
            parameters: serde_json::json!({"query": "string"}),
        }];

        let prompt = client.build_react_prompt("What is Rust?", &tools, "", None);

        assert!(prompt.contains("search"));
        assert!(prompt.contains("What is Rust?"));
        assert!(prompt.contains("ReAct"));
    }
}
