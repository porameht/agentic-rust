//! ReAct (Reasoning + Acting) Agent Implementation
//!
//! This module implements the ReAct pattern for Flow 1, providing an iterative
//! reasoning loop where the agent can think, act (use tools), and observe results.
//!
//! # ReAct Pattern
//!
//! ```text
//! User Query
//!     ↓
//! ┌──────────────────────────────────────┐
//! │  REASONING LOOP (max_iterations)     │
//! │                                      │
//! │  1. THINK: Analyze the task         │
//! │     "I need to search for..."       │
//! │           ↓                          │
//! │  2. ACT: Choose and execute tool    │
//! │     Action: search(query)           │
//! │           ↓                          │
//! │  3. OBSERVE: Process tool result    │
//! │     Observation: "Found..."         │
//! │           ↓                          │
//! │  4. CHECK: Is task complete?        │
//! │     ├─ NO  → Loop back to THINK     │
//! │     └─ YES → Output Final Answer    │
//! └──────────────────────────────────────┘
//!     ↓
//! Final Response + Reasoning Trace
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use agent::react::{ReActAgent, ReActConfig};
//!
//! let config = ReActConfig::builder()
//!     .model("gpt-4")
//!     .max_iterations(10)
//!     .temperature(0.3)
//!     .build();
//!
//! let agent = ReActAgent::new(config)
//!     .with_tool(SearchTool::new())
//!     .with_tool(CalculatorTool::new());
//!
//! let response = agent.run("What is the population of Tokyo?").await?;
//! println!("Answer: {}", response.final_answer);
//! println!("Reasoning steps: {:?}", response.trace);
//! ```

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::tools::{Tool, ToolDefinition, ToolResult};

// ============================================================================
// ERRORS
// ============================================================================

/// Errors that can occur during ReAct agent execution
#[derive(Error, Debug)]
pub enum ReActError {
    #[error("Maximum iterations ({0}) exceeded without reaching final answer")]
    MaxIterationsExceeded(usize),

    #[error("Tool '{0}' not found in registry")]
    ToolNotFound(String),

    #[error("Tool execution failed: {0}")]
    ToolExecutionFailed(String),

    #[error("LLM call failed: {0}")]
    LlmError(String),

    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Context retrieval failed: {0}")]
    RetrievalError(String),
}

// ============================================================================
// STATE MACHINE
// ============================================================================

/// The current state of the ReAct agent
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReActState {
    /// Agent is ready to start processing
    Ready,

    /// Agent is thinking about what to do next
    Thinking,

    /// Agent has decided on an action and is executing a tool
    Acting { tool_name: String },

    /// Agent is processing the observation from a tool
    Observing,

    /// Agent has reached a final answer
    Finished,

    /// Agent encountered an error
    Error { message: String },
}

impl std::fmt::Display for ReActState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReActState::Ready => write!(f, "Ready"),
            ReActState::Thinking => write!(f, "Thinking"),
            ReActState::Acting { tool_name } => write!(f, "Acting({})", tool_name),
            ReActState::Observing => write!(f, "Observing"),
            ReActState::Finished => write!(f, "Finished"),
            ReActState::Error { message } => write!(f, "Error: {}", message),
        }
    }
}

// ============================================================================
// THOUGHT/ACTION PARSING
// ============================================================================

/// Parsed output from the LLM during ReAct loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThoughtAction {
    /// Agent is thinking about the problem
    Thought { content: String },

    /// Agent wants to use a tool
    Action {
        tool_name: String,
        tool_input: serde_json::Value,
        thought: Option<String>,
    },

    /// Agent has reached a final answer
    FinalAnswer {
        answer: String,
        thought: Option<String>,
    },
}

impl ThoughtAction {
    /// Parse LLM output into ThoughtAction
    ///
    /// Expected formats:
    /// - Thought: <thinking>
    /// - Action: <tool_name>
    ///   Action Input: <json>
    /// - Final Answer: <answer>
    pub fn parse(text: &str) -> Result<Self, ReActError> {
        let text = text.trim();

        // Check for Final Answer first
        if let Some(answer) = Self::extract_final_answer(text) {
            let thought = Self::extract_thought(text);
            return Ok(ThoughtAction::FinalAnswer { answer, thought });
        }

        // Check for Action
        if let Some((tool_name, tool_input, thought)) = Self::extract_action(text) {
            return Ok(ThoughtAction::Action {
                tool_name,
                tool_input,
                thought,
            });
        }

        // Otherwise it's just a thought
        let thought = Self::extract_thought(text).unwrap_or_else(|| text.to_string());
        Ok(ThoughtAction::Thought { content: thought })
    }

    fn extract_thought(text: &str) -> Option<String> {
        // Look for "Thought:" prefix
        for line in text.lines() {
            let line = line.trim();
            if line.to_lowercase().starts_with("thought:") {
                return Some(line[8..].trim().to_string());
            }
        }
        None
    }

    fn extract_final_answer(text: &str) -> Option<String> {
        // Look for "Final Answer:" prefix
        let lower = text.to_lowercase();
        if let Some(idx) = lower.find("final answer:") {
            let start = idx + "final answer:".len();
            return Some(text[start..].trim().to_string());
        }
        None
    }

    fn extract_action(text: &str) -> Option<(String, serde_json::Value, Option<String>)> {
        let thought = Self::extract_thought(text);

        // Look for "Action:" and "Action Input:"
        let lower = text.to_lowercase();

        let action_idx = lower.find("action:")?;
        let action_start = action_idx + "action:".len();

        // Find the end of the action line
        let remaining = &text[action_start..];
        let action_end = remaining.find('\n').unwrap_or(remaining.len());
        let tool_name = remaining[..action_end].trim().to_string();

        // Skip if it's "Action Input:" on the same line somehow
        if tool_name.to_lowercase().starts_with("input") {
            return None;
        }

        // Look for "Action Input:"
        let input_marker = "action input:";
        let input_idx = lower.find(input_marker)?;
        let input_start = input_idx + input_marker.len();
        let input_text = text[input_start..].trim();

        // Try to parse as JSON
        let tool_input = if input_text.starts_with('{') || input_text.starts_with('[') {
            // Find the end of the JSON
            let json_text = Self::extract_json(input_text)?;
            serde_json::from_str(&json_text).ok()?
        } else {
            // Treat as a simple string query
            serde_json::json!({ "query": input_text.lines().next().unwrap_or(input_text).trim() })
        };

        Some((tool_name, tool_input, thought))
    }

    fn extract_json(text: &str) -> Option<String> {
        let mut depth = 0;
        let mut in_string = false;
        let mut escape_next = false;
        let mut end_idx = 0;

        for (i, ch) in text.char_indices() {
            if escape_next {
                escape_next = false;
                continue;
            }

            match ch {
                '\\' if in_string => escape_next = true,
                '"' => in_string = !in_string,
                '{' | '[' if !in_string => depth += 1,
                '}' | ']' if !in_string => {
                    depth -= 1;
                    if depth == 0 {
                        end_idx = i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if end_idx > 0 {
            Some(text[..end_idx].to_string())
        } else {
            None
        }
    }
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Configuration for the ReAct agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActConfig {
    /// Model to use (e.g., "gpt-4", "claude-3-opus")
    pub model: String,

    /// System preamble for the agent
    pub preamble: String,

    /// Maximum number of reasoning iterations
    pub max_iterations: usize,

    /// Temperature for LLM calls (0.0 - 1.0)
    pub temperature: f32,

    /// Whether to include RAG context
    pub use_rag: bool,

    /// Number of documents to retrieve for RAG
    pub top_k_documents: usize,

    /// Whether to return the full reasoning trace
    pub return_trace: bool,

    /// Timeout for each iteration in seconds
    pub iteration_timeout_secs: Option<u64>,
}

impl Default for ReActConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4".to_string(),
            preamble: DEFAULT_REACT_PREAMBLE.to_string(),
            max_iterations: 10,
            temperature: 0.3,
            use_rag: true,
            top_k_documents: 5,
            return_trace: true,
            iteration_timeout_secs: Some(30),
        }
    }
}

impl ReActConfig {
    /// Create a new builder
    pub fn builder() -> ReActConfigBuilder {
        ReActConfigBuilder::default()
    }
}

/// Builder for ReActConfig
#[derive(Default)]
pub struct ReActConfigBuilder {
    config: ReActConfig,
}

impl ReActConfigBuilder {
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.config.preamble = preamble.into();
        self
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = temp;
        self
    }

    pub fn use_rag(mut self, use_rag: bool) -> Self {
        self.config.use_rag = use_rag;
        self
    }

    pub fn top_k_documents(mut self, k: usize) -> Self {
        self.config.top_k_documents = k;
        self
    }

    pub fn return_trace(mut self, return_trace: bool) -> Self {
        self.config.return_trace = return_trace;
        self
    }

    pub fn iteration_timeout(mut self, secs: u64) -> Self {
        self.config.iteration_timeout_secs = Some(secs);
        self
    }

    pub fn build(self) -> ReActConfig {
        self.config
    }
}

// ============================================================================
// REASONING TRACE
// ============================================================================

/// A single step in the reasoning trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActStep {
    /// Step number (1-indexed)
    pub step: usize,

    /// State at this step
    pub state: ReActState,

    /// The thought/reasoning at this step
    pub thought: Option<String>,

    /// Action taken (if any)
    pub action: Option<ActionRecord>,

    /// Observation received (if any)
    pub observation: Option<String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Duration of this step in milliseconds
    pub duration_ms: u64,
}

/// Record of an action taken
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecord {
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    pub tool_output: Option<String>,
    pub success: bool,
}

// ============================================================================
// RESPONSE
// ============================================================================

/// Response from the ReAct agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReActResponse {
    /// Unique ID for this execution
    pub id: Uuid,

    /// The final answer
    pub final_answer: String,

    /// Number of iterations taken
    pub iterations: usize,

    /// Total execution time in milliseconds
    pub total_duration_ms: u64,

    /// The reasoning trace (if return_trace is true)
    pub trace: Option<Vec<ReActStep>>,

    /// Sources from RAG (if used)
    pub sources: Vec<String>,

    /// Final state
    pub state: ReActState,

    /// Token usage statistics
    pub token_usage: Option<TokenUsage>,
}

/// Token usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// ============================================================================
// REACT AGENT
// ============================================================================

/// ReAct Agent with tool support
///
/// This agent implements the ReAct (Reasoning + Acting) pattern,
/// iteratively thinking about problems and using tools to gather information.
pub struct ReActAgent {
    config: ReActConfig,
    tools: HashMap<String, Arc<dyn Tool>>,
    state: ReActState,
}

impl ReActAgent {
    /// Create a new ReAct agent with the given configuration
    pub fn new(config: ReActConfig) -> Self {
        Self {
            config,
            tools: HashMap::new(),
            state: ReActState::Ready,
        }
    }

    /// Add a tool to the agent
    pub fn with_tool<T: Tool + 'static>(mut self, tool: T) -> Self {
        let name = tool.definition().name.clone();
        self.tools.insert(name, Arc::new(tool));
        self
    }

    /// Add multiple tools
    pub fn with_tools(mut self, tools: Vec<Box<dyn Tool>>) -> Self {
        for tool in tools {
            let name = tool.definition().name.clone();
            self.tools.insert(name, Arc::from(tool));
        }
        self
    }

    /// Get the current state
    pub fn state(&self) -> &ReActState {
        &self.state
    }

    /// Get the configuration
    pub fn config(&self) -> &ReActConfig {
        &self.config
    }

    /// Get available tool definitions
    pub fn tool_definitions(&self) -> Vec<ToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Execute the ReAct loop for a given query
    pub async fn run(&mut self, query: &str) -> Result<ReActResponse, ReActError> {
        self.run_with_context(query, None).await
    }

    /// Execute the ReAct loop with optional RAG context
    pub async fn run_with_context(
        &mut self,
        query: &str,
        context: Option<Vec<String>>,
    ) -> Result<ReActResponse, ReActError> {
        let start_time = std::time::Instant::now();
        let execution_id = Uuid::new_v4();

        info!(
            execution_id = %execution_id,
            query = %query,
            "Starting ReAct execution"
        );

        self.state = ReActState::Ready;
        let mut trace = Vec::new();
        let mut scratchpad = String::new();
        let mut iteration = 0;
        let total_tokens = TokenUsage::default();

        // Build initial prompt with tools
        let tool_descriptions = self.format_tool_descriptions();
        let context_str = context
            .as_ref()
            .map(|c| format!("\n\nContext:\n{}", c.join("\n\n")))
            .unwrap_or_default();

        loop {
            iteration += 1;
            let step_start = std::time::Instant::now();

            if iteration > self.config.max_iterations {
                self.state = ReActState::Error {
                    message: format!("Max iterations ({}) exceeded", self.config.max_iterations),
                };
                return Err(ReActError::MaxIterationsExceeded(self.config.max_iterations));
            }

            // THINK: Generate next thought/action
            self.state = ReActState::Thinking;
            debug!(iteration = iteration, "Thinking...");

            let prompt = self.build_prompt(query, &tool_descriptions, &context_str, &scratchpad);
            let llm_response = self.call_llm(&prompt).await?;

            // Parse the response
            let thought_action = ThoughtAction::parse(&llm_response)?;

            match thought_action {
                ThoughtAction::Thought { content } => {
                    // Just thinking, add to scratchpad and continue
                    scratchpad.push_str(&format!("\nThought: {}", content));

                    trace.push(ReActStep {
                        step: iteration,
                        state: ReActState::Thinking,
                        thought: Some(content),
                        action: None,
                        observation: None,
                        timestamp: Utc::now(),
                        duration_ms: step_start.elapsed().as_millis() as u64,
                    });
                }

                ThoughtAction::Action {
                    tool_name,
                    tool_input,
                    thought,
                } => {
                    // ACT: Execute the tool
                    self.state = ReActState::Acting {
                        tool_name: tool_name.clone(),
                    };
                    debug!(iteration = iteration, tool = %tool_name, "Acting...");

                    if let Some(t) = &thought {
                        scratchpad.push_str(&format!("\nThought: {}", t));
                    }
                    scratchpad.push_str(&format!("\nAction: {}", tool_name));
                    scratchpad.push_str(&format!("\nAction Input: {}", tool_input));

                    // Execute tool
                    let tool_result = self.execute_tool(&tool_name, tool_input.clone()).await;

                    // OBSERVE: Process the result
                    self.state = ReActState::Observing;
                    let (observation, success) = match &tool_result {
                        Ok(result) => (result.output.clone(), result.success),
                        Err(e) => (format!("Error: {}", e), false),
                    };

                    scratchpad.push_str(&format!("\nObservation: {}", observation));
                    debug!(iteration = iteration, "Observation: {}", observation);

                    trace.push(ReActStep {
                        step: iteration,
                        state: ReActState::Acting {
                            tool_name: tool_name.clone(),
                        },
                        thought,
                        action: Some(ActionRecord {
                            tool_name,
                            tool_input,
                            tool_output: Some(observation.clone()),
                            success,
                        }),
                        observation: Some(observation),
                        timestamp: Utc::now(),
                        duration_ms: step_start.elapsed().as_millis() as u64,
                    });
                }

                ThoughtAction::FinalAnswer { answer, thought } => {
                    // FINISH: We have a final answer
                    self.state = ReActState::Finished;
                    info!(
                        execution_id = %execution_id,
                        iterations = iteration,
                        "ReAct execution completed"
                    );

                    if let Some(t) = &thought {
                        scratchpad.push_str(&format!("\nThought: {}", t));
                    }

                    trace.push(ReActStep {
                        step: iteration,
                        state: ReActState::Finished,
                        thought,
                        action: None,
                        observation: None,
                        timestamp: Utc::now(),
                        duration_ms: step_start.elapsed().as_millis() as u64,
                    });

                    return Ok(ReActResponse {
                        id: execution_id,
                        final_answer: answer,
                        iterations: iteration,
                        total_duration_ms: start_time.elapsed().as_millis() as u64,
                        trace: if self.config.return_trace {
                            Some(trace)
                        } else {
                            None
                        },
                        sources: context.unwrap_or_default(),
                        state: ReActState::Finished,
                        token_usage: Some(total_tokens),
                    });
                }
            }
        }
    }

    /// Build the prompt for the LLM
    fn build_prompt(
        &self,
        query: &str,
        tool_descriptions: &str,
        context: &str,
        scratchpad: &str,
    ) -> String {
        format!(
            r#"{preamble}

{tool_descriptions}
{context}

Use the following format:

Question: the input question you must answer
Thought: you should always think about what to do
Action: the action to take, should be one of [{tool_names}]
Action Input: the input to the action (as JSON)
Observation: the result of the action
... (this Thought/Action/Action Input/Observation can repeat N times)
Thought: I now know the final answer
Final Answer: the final answer to the original input question

Begin!

Question: {query}
{scratchpad}"#,
            preamble = self.config.preamble,
            tool_descriptions = tool_descriptions,
            context = context,
            tool_names = self.tools.keys().cloned().collect::<Vec<_>>().join(", "),
            query = query,
            scratchpad = scratchpad,
        )
    }

    /// Format tool descriptions for the prompt
    fn format_tool_descriptions(&self) -> String {
        if self.tools.is_empty() {
            return "You have no tools available. Answer based on your knowledge.".to_string();
        }

        let mut desc = String::from("You have access to the following tools:\n\n");
        for tool in self.tools.values() {
            let def = tool.definition();
            desc.push_str(&format!(
                "- {}: {}\n  Parameters: {}\n\n",
                def.name,
                def.description,
                serde_json::to_string_pretty(&def.parameters).unwrap_or_default()
            ));
        }
        desc
    }

    /// Call the LLM with the given prompt
    async fn call_llm(&self, _prompt: &str) -> Result<String, ReActError> {
        // TODO: Integrate with rig's completion API
        // Example implementation:
        //
        // use rig::providers::openai;
        // let client = openai::Client::from_env();
        // let completion = client
        //     .completion(&self.config.model)
        //     .temperature(self.config.temperature)
        //     .build();
        //
        // completion
        //     .prompt(prompt)
        //     .await
        //     .map_err(|e| ReActError::LlmError(e.to_string()))

        // Placeholder: Return a mock response for testing
        warn!("LLM integration not implemented - returning mock response");
        Ok(format!(
            "Thought: This is a placeholder response. LLM integration pending.\n\
             Final Answer: [Mock] The ReAct agent received your query but LLM is not yet integrated."
        ))
    }

    /// Execute a tool by name
    async fn execute_tool(
        &self,
        tool_name: &str,
        input: serde_json::Value,
    ) -> Result<ToolResult, ReActError> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| ReActError::ToolNotFound(tool_name.to_string()))?;

        tool.execute(input)
            .await
            .map_err(|e| ReActError::ToolExecutionFailed(e.to_string()))
    }
}

// ============================================================================
// DEFAULT PROMPTS
// ============================================================================

/// Default system preamble for ReAct agent
pub const DEFAULT_REACT_PREAMBLE: &str = r#"You are a helpful AI assistant that uses reasoning and tools to answer questions accurately.

When given a question, you will:
1. Think step by step about what you need to do
2. Use available tools to gather information when needed
3. Observe the results and continue reasoning
4. Provide a clear, accurate final answer

Always be thorough but efficient. Only use tools when necessary.
If you can answer from your knowledge without tools, do so directly."#;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thought_action_parse_final_answer() {
        let text = "Thought: I have enough information now.\nFinal Answer: The answer is 42.";
        let result = ThoughtAction::parse(text).unwrap();

        match result {
            ThoughtAction::FinalAnswer { answer, thought } => {
                assert_eq!(answer, "The answer is 42.");
                assert_eq!(thought, Some("I have enough information now.".to_string()));
            }
            _ => panic!("Expected FinalAnswer"),
        }
    }

    #[test]
    fn test_thought_action_parse_action() {
        let text = r#"Thought: I need to search for this.
Action: search
Action Input: {"query": "rust programming"}"#;

        let result = ThoughtAction::parse(text).unwrap();

        match result {
            ThoughtAction::Action {
                tool_name,
                tool_input,
                thought,
            } => {
                assert_eq!(tool_name, "search");
                assert_eq!(tool_input["query"], "rust programming");
                assert!(thought.is_some());
            }
            _ => panic!("Expected Action"),
        }
    }

    #[test]
    fn test_thought_action_parse_thought_only() {
        let text = "Thought: I need to think about this more carefully.";
        let result = ThoughtAction::parse(text).unwrap();

        match result {
            ThoughtAction::Thought { content } => {
                assert_eq!(content, "I need to think about this more carefully.");
            }
            _ => panic!("Expected Thought"),
        }
    }

    #[test]
    fn test_react_config_builder() {
        let config = ReActConfig::builder()
            .model("claude-3-opus")
            .max_iterations(5)
            .temperature(0.5)
            .use_rag(false)
            .build();

        assert_eq!(config.model, "claude-3-opus");
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.temperature, 0.5);
        assert!(!config.use_rag);
    }

    #[test]
    fn test_react_state_display() {
        assert_eq!(ReActState::Ready.to_string(), "Ready");
        assert_eq!(ReActState::Thinking.to_string(), "Thinking");
        assert_eq!(
            ReActState::Acting {
                tool_name: "search".to_string()
            }
            .to_string(),
            "Acting(search)"
        );
        assert_eq!(ReActState::Finished.to_string(), "Finished");
    }

    #[test]
    fn test_extract_json() {
        let text = r#"{"query": "test", "limit": 5} some trailing text"#;
        let json = ThoughtAction::extract_json(text).unwrap();
        assert_eq!(json, r#"{"query": "test", "limit": 5}"#);
    }

    #[test]
    fn test_extract_nested_json() {
        let text = r#"{"outer": {"inner": "value"}, "array": [1, 2, 3]}"#;
        let json = ThoughtAction::extract_json(text).unwrap();
        assert_eq!(json, text);
    }

    #[tokio::test]
    async fn test_react_agent_creation() {
        let config = ReActConfig::default();
        let agent = ReActAgent::new(config);

        assert_eq!(*agent.state(), ReActState::Ready);
        assert!(agent.tools.is_empty());
    }

    #[test]
    fn test_format_tool_descriptions_empty() {
        let agent = ReActAgent::new(ReActConfig::default());
        let desc = agent.format_tool_descriptions();
        assert!(desc.contains("no tools available"));
    }
}
