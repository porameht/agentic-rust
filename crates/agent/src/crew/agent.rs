//! CrewAI-style Agent Implementation
//!
//! Agents are autonomous units with specific roles, goals, and capabilities.
//! Each agent has a distinct personality defined by its backstory and can
//! use tools to accomplish tasks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

use super::memory::{Memory, MemoryConfig, MemoryType};
use crate::tools::{Tool, ToolDefinition, ToolResult};

/// Errors that can occur during agent operations
#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Agent execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool not found: {0}")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    #[error("LLM provider error: {0}")]
    LlmError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),
}

/// Configuration for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique identifier for the agent
    pub id: String,

    /// The role of the agent (e.g., "Senior Research Analyst")
    pub role: String,

    /// The primary goal the agent is trying to achieve
    pub goal: String,

    /// Background story providing personality and context
    pub backstory: String,

    /// LLM model to use (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// Temperature for LLM responses (0.0 - 1.0)
    pub temperature: f32,

    /// Maximum tokens for LLM responses
    pub max_tokens: Option<usize>,

    /// Whether the agent can delegate tasks to other agents
    pub allow_delegation: bool,

    /// Whether verbose logging is enabled
    pub verbose: bool,

    /// Maximum number of iterations for task execution
    pub max_iterations: usize,

    /// Maximum execution time in seconds
    pub max_execution_time: Option<u64>,

    /// Names of tools available to this agent
    pub tools: Vec<String>,

    /// Memory configuration
    pub memory_config: Option<MemoryConfig>,

    /// Custom system prompt additions
    pub system_prompt_suffix: Option<String>,

    /// Response format hints
    pub response_format: Option<String>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            role: String::new(),
            goal: String::new(),
            backstory: String::new(),
            model: "gpt-4".to_string(),
            temperature: 0.7,
            max_tokens: None,
            allow_delegation: false,
            verbose: false,
            max_iterations: 10,
            max_execution_time: Some(300), // 5 minutes default
            tools: Vec::new(),
            memory_config: None,
            system_prompt_suffix: None,
            response_format: None,
            metadata: HashMap::new(),
        }
    }
}

/// Execution context passed to agents during task execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Task description
    pub task_description: String,

    /// Expected output format
    pub expected_output: String,

    /// Context from previous tasks
    pub context: Vec<String>,

    /// Available tools
    pub available_tools: Vec<ToolDefinition>,

    /// Shared crew memory/state
    pub shared_state: HashMap<String, serde_json::Value>,

    /// Current iteration number
    pub iteration: usize,

    /// Maximum allowed iterations
    pub max_iterations: usize,
}

/// Result of an agent's execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionResult {
    /// The agent's output/response
    pub output: String,

    /// Any tool calls made during execution
    pub tool_calls: Vec<ToolCallRecord>,

    /// Reasoning/thought process (if verbose)
    pub reasoning: Option<String>,

    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Record of a tool call made by an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    /// Tool name
    pub tool_name: String,

    /// Tool arguments
    pub arguments: serde_json::Value,

    /// Tool result
    pub result: String,

    /// Whether the call was successful
    pub success: bool,

    /// Timestamp of the call
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Metadata about agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Total iterations used
    pub iterations: usize,

    /// Total execution time in milliseconds
    pub execution_time_ms: u64,

    /// Number of LLM calls made
    pub llm_calls: usize,

    /// Token usage (if available)
    pub tokens_used: Option<TokenUsage>,

    /// Timestamp when execution started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when execution completed
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

/// Trait for agent execution behavior
#[async_trait]
pub trait AgentExecutor: Send + Sync {
    /// Execute the agent's task
    async fn execute(&self, context: ExecutionContext) -> Result<AgentExecutionResult, AgentError>;

    /// Get the agent's configuration
    fn config(&self) -> &AgentConfig;

    /// Generate the system prompt for this agent
    fn system_prompt(&self) -> String {
        let config = self.config();
        let mut prompt = format!(
            "You are a {}.\n\n\
             Your goal is: {}\n\n\
             Background:\n{}\n",
            config.role, config.goal, config.backstory
        );

        if !config.tools.is_empty() {
            prompt.push_str("\nYou have access to the following tools:\n");
            for tool in &config.tools {
                prompt.push_str(&format!("- {}\n", tool));
            }
        }

        if let Some(suffix) = &config.system_prompt_suffix {
            prompt.push_str("\n");
            prompt.push_str(suffix);
        }

        prompt
    }
}

/// CrewAI-style Agent
pub struct Agent {
    /// Agent configuration
    config: AgentConfig,

    /// Available tools (Arc for shared ownership)
    tools: HashMap<String, Arc<dyn Tool>>,

    /// Agent memory
    memory: Option<Memory>,
}

impl Agent {
    /// Create a new agent builder
    pub fn builder() -> AgentBuilder {
        AgentBuilder::new()
    }

    /// Create a new agent with the given configuration
    pub fn new(config: AgentConfig) -> Self {
        let memory = config
            .memory_config
            .as_ref()
            .map(|mc| Memory::new(mc.clone()));

        Self {
            config,
            tools: HashMap::new(),
            memory,
        }
    }

    /// Get the agent's unique identifier
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get the agent's role
    pub fn role(&self) -> &str {
        &self.config.role
    }

    /// Get the agent's goal
    pub fn goal(&self) -> &str {
        &self.config.goal
    }

    /// Get the agent's backstory
    pub fn backstory(&self) -> &str {
        &self.config.backstory
    }

    /// Get the agent's model
    pub fn model(&self) -> &str {
        &self.config.model
    }

    /// Check if delegation is allowed
    pub fn allows_delegation(&self) -> bool {
        self.config.allow_delegation
    }

    /// Add a tool to the agent
    pub fn add_tool(&mut self, tool: Arc<dyn Tool>) {
        let definition = tool.definition();
        self.config.tools.push(definition.name.clone());
        self.tools.insert(definition.name, tool);
    }

    /// Get tool definitions
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute a tool by name
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        args: serde_json::Value,
    ) -> Result<ToolResult, AgentError> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| AgentError::ToolNotFound(tool_name.to_string()))?;

        tool.execute(args)
            .await
            .map_err(|e| AgentError::ToolExecutionFailed(e.to_string()))
    }

    /// Store information in memory
    pub async fn remember(&self, key: &str, value: serde_json::Value) -> Result<(), AgentError> {
        if let Some(memory) = &self.memory {
            memory
                .store(key, value)
                .await
                .map_err(|e| AgentError::MemoryError(e.to_string()))
        } else {
            Ok(()) // No-op if no memory configured
        }
    }

    /// Retrieve information from memory
    pub async fn recall(&self, key: &str) -> Result<Option<serde_json::Value>, AgentError> {
        if let Some(memory) = &self.memory {
            memory
                .retrieve(key)
                .await
                .map_err(|e| AgentError::MemoryError(e.to_string()))
        } else {
            Ok(None)
        }
    }
}

#[async_trait]
impl AgentExecutor for Agent {
    async fn execute(&self, context: ExecutionContext) -> Result<AgentExecutionResult, AgentError> {
        let started_at = chrono::Utc::now();
        let mut iterations = 0;
        let tool_calls = Vec::new();
        let mut reasoning = if self.config.verbose {
            Some(String::new())
        } else {
            None
        };

        // Build the prompt with context
        let mut prompt = format!(
            "Task: {}\n\nExpected Output: {}",
            context.task_description, context.expected_output
        );

        if !context.context.is_empty() {
            prompt.push_str("\n\nContext from previous tasks:\n");
            for (i, ctx) in context.context.iter().enumerate() {
                prompt.push_str(&format!("--- Context {} ---\n{}\n", i + 1, ctx));
            }
        }

        // TODO: Integrate with rig for actual LLM calls
        // For now, this is a placeholder that demonstrates the structure
        let output = format!(
            "[Agent: {}] Processed task: {}\n\
             Based on my role as {}, I would approach this by:\n\
             1. Analyzing the requirements\n\
             2. Applying my expertise from: {}\n\
             3. Delivering output matching: {}",
            self.config.id,
            context.task_description,
            self.config.role,
            self.config.backstory,
            context.expected_output
        );

        if let Some(ref mut r) = reasoning {
            r.push_str(&format!(
                "Agent {} reasoning:\n- Received task description\n- \
                 Applied role context\n- Generated response\n",
                self.config.id
            ));
        }

        iterations += 1;
        let completed_at = chrono::Utc::now();
        let execution_time_ms = (completed_at - started_at).num_milliseconds() as u64;

        Ok(AgentExecutionResult {
            output,
            tool_calls,
            reasoning,
            metadata: ExecutionMetadata {
                iterations,
                execution_time_ms,
                llm_calls: 1,
                tokens_used: None,
                started_at,
                completed_at,
            },
        })
    }

    fn config(&self) -> &AgentConfig {
        &self.config
    }
}

/// Builder for creating agents with a fluent API
pub struct AgentBuilder {
    config: AgentConfig,
    tools: Vec<Arc<dyn Tool>>,
}

impl AgentBuilder {
    /// Create a new agent builder
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
            tools: Vec::new(),
        }
    }

    /// Set the agent's unique identifier
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.config.id = id.into();
        self
    }

    /// Set the agent's role
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.config.role = role.into();
        self
    }

    /// Set the agent's goal
    pub fn goal(mut self, goal: impl Into<String>) -> Self {
        self.config.goal = goal.into();
        self
    }

    /// Set the agent's backstory
    pub fn backstory(mut self, backstory: impl Into<String>) -> Self {
        self.config.backstory = backstory.into();
        self
    }

    /// Set the LLM model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Set maximum tokens
    pub fn max_tokens(mut self, max_tokens: usize) -> Self {
        self.config.max_tokens = Some(max_tokens);
        self
    }

    /// Allow or disallow delegation
    pub fn allow_delegation(mut self, allow: bool) -> Self {
        self.config.allow_delegation = allow;
        self
    }

    /// Enable or disable verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    /// Set maximum iterations
    pub fn max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    /// Set maximum execution time in seconds
    pub fn max_execution_time(mut self, seconds: u64) -> Self {
        self.config.max_execution_time = Some(seconds);
        self
    }

    /// Add a tool by name
    pub fn tool_name(mut self, tool_name: impl Into<String>) -> Self {
        self.config.tools.push(tool_name.into());
        self
    }

    /// Add a tool instance
    pub fn tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }

    /// Configure memory
    pub fn memory(mut self, config: MemoryConfig) -> Self {
        self.config.memory_config = Some(config);
        self
    }

    /// Enable short-term memory
    pub fn with_short_term_memory(mut self) -> Self {
        self.config.memory_config = Some(MemoryConfig {
            memory_type: MemoryType::ShortTerm,
            ..Default::default()
        });
        self
    }

    /// Enable long-term memory
    pub fn with_long_term_memory(mut self) -> Self {
        self.config.memory_config = Some(MemoryConfig {
            memory_type: MemoryType::LongTerm,
            ..Default::default()
        });
        self
    }

    /// Set custom system prompt suffix
    pub fn system_prompt_suffix(mut self, suffix: impl Into<String>) -> Self {
        self.config.system_prompt_suffix = Some(suffix.into());
        self
    }

    /// Set response format
    pub fn response_format(mut self, format: impl Into<String>) -> Self {
        self.config.response_format = Some(format.into());
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.metadata.insert(key.into(), value);
        self
    }

    /// Build the agent
    pub fn build(self) -> Agent {
        let mut agent = Agent::new(self.config);
        for tool in self.tools {
            agent.add_tool(tool);
        }
        agent
    }
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder() {
        let agent = Agent::builder()
            .id("test-agent")
            .role("Test Role")
            .goal("Test Goal")
            .backstory("Test Backstory")
            .model("gpt-4")
            .temperature(0.5)
            .verbose(true)
            .build();

        assert_eq!(agent.id(), "test-agent");
        assert_eq!(agent.role(), "Test Role");
        assert_eq!(agent.goal(), "Test Goal");
        assert_eq!(agent.backstory(), "Test Backstory");
        assert_eq!(agent.model(), "gpt-4");
    }

    #[test]
    fn test_system_prompt_generation() {
        let agent = Agent::builder()
            .role("Senior Analyst")
            .goal("Analyze data and provide insights")
            .backstory("Expert with 10 years of experience")
            .build();

        let prompt = agent.system_prompt();
        assert!(prompt.contains("Senior Analyst"));
        assert!(prompt.contains("Analyze data and provide insights"));
        assert!(prompt.contains("10 years of experience"));
    }
}
