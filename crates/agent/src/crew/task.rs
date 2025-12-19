//! CrewAI-style Task Implementation
//!
//! Tasks are specific assignments given to agents. They define what needs
//! to be done, the expected output, and can depend on other tasks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during task operations
#[derive(Error, Debug)]
pub enum TaskError {
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Task dependency not met: {0}")]
    DependencyNotMet(String),

    #[error("No agent assigned to task")]
    NoAgentAssigned,

    #[error("Task timeout after {0} seconds")]
    Timeout(u64),

    #[error("Task validation failed: {0}")]
    ValidationFailed(String),

    #[error("Callback error: {0}")]
    CallbackError(String),
}

/// Status of a task
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Task is waiting to be executed
    Pending,
    /// Task is currently being executed
    InProgress,
    /// Task completed successfully
    Completed,
    /// Task failed
    Failed,
    /// Task was skipped
    Skipped,
    /// Task was cancelled
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Configuration for a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConfig {
    /// Unique identifier for the task
    pub id: String,

    /// Human-readable name for the task
    pub name: Option<String>,

    /// Detailed description of what the task should accomplish
    pub description: String,

    /// Description of the expected output format and content
    pub expected_output: String,

    /// ID of the agent assigned to this task
    pub agent_id: Option<String>,

    /// IDs of tasks that must complete before this task
    pub dependencies: Vec<String>,

    /// Whether this task is asynchronous
    pub is_async: bool,

    /// Maximum execution time in seconds
    pub timeout: Option<u64>,

    /// Whether to use human feedback for validation
    pub human_input: bool,

    /// Output file path (if task should write to file)
    pub output_file: Option<String>,

    /// Specific tools this task should use
    pub tools: Vec<String>,

    /// Additional context or instructions
    pub context_instructions: Option<String>,

    /// Whether the task output should be included in crew's final output
    pub include_in_output: bool,

    /// Retry configuration
    pub max_retries: usize,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: None,
            description: String::new(),
            expected_output: String::new(),
            agent_id: None,
            dependencies: Vec::new(),
            is_async: false,
            timeout: None,
            human_input: false,
            output_file: None,
            tools: Vec::new(),
            context_instructions: None,
            include_in_output: true,
            max_retries: 0,
            metadata: HashMap::new(),
        }
    }
}

/// Context provided from other tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    /// ID of the source task
    pub source_task_id: String,

    /// Output from the source task
    pub output: String,

    /// Whether this was a successful execution
    pub success: bool,

    /// Timestamp when the source task completed
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Output from a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutput {
    /// The main output/result
    pub result: String,

    /// Raw output before any processing
    pub raw_output: String,

    /// Whether the output was exported to a file
    pub exported_to_file: Option<String>,

    /// Summary of the output (for context passing)
    pub summary: Option<String>,

    /// Structured data extracted from the output
    pub structured_data: Option<serde_json::Value>,

    /// Any warnings or notes
    pub notes: Vec<String>,
}

impl TaskOutput {
    /// Create a new task output
    pub fn new(result: impl Into<String>) -> Self {
        let result = result.into();
        Self {
            raw_output: result.clone(),
            result,
            exported_to_file: None,
            summary: None,
            structured_data: None,
            notes: Vec::new(),
        }
    }

    /// Add a summary
    pub fn with_summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = Some(summary.into());
        self
    }

    /// Add structured data
    pub fn with_structured_data(mut self, data: serde_json::Value) -> Self {
        self.structured_data = Some(data);
        self
    }

    /// Add a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

/// A task to be executed by an agent
#[derive(Debug, Clone)]
pub struct Task {
    /// Task configuration
    config: TaskConfig,

    /// Current status
    status: TaskStatus,

    /// Output from execution (if completed)
    output: Option<TaskOutput>,

    /// Context from dependent tasks
    context: Vec<TaskContext>,

    /// Error message (if failed)
    error: Option<String>,

    /// Execution attempt count
    attempts: usize,

    /// Timestamps
    created_at: chrono::DateTime<chrono::Utc>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Task {
    /// Create a new task builder
    pub fn builder() -> TaskBuilder {
        TaskBuilder::new()
    }

    /// Create a new task with configuration
    pub fn new(config: TaskConfig) -> Self {
        Self {
            config,
            status: TaskStatus::Pending,
            output: None,
            context: Vec::new(),
            error: None,
            attempts: 0,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    /// Get the task ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get the task name
    pub fn name(&self) -> Option<&str> {
        self.config.name.as_deref()
    }

    /// Get the task description
    pub fn description(&self) -> &str {
        &self.config.description
    }

    /// Get the expected output
    pub fn expected_output(&self) -> &str {
        &self.config.expected_output
    }

    /// Get the assigned agent ID
    pub fn agent_id(&self) -> Option<&str> {
        self.config.agent_id.as_deref()
    }

    /// Get task dependencies
    pub fn dependencies(&self) -> &[String] {
        &self.config.dependencies
    }

    /// Check if task is async
    pub fn is_async(&self) -> bool {
        self.config.is_async
    }

    /// Get current status
    pub fn status(&self) -> TaskStatus {
        self.status
    }

    /// Get task output (if completed)
    pub fn output(&self) -> Option<&TaskOutput> {
        self.output.as_ref()
    }

    /// Get task context
    pub fn context(&self) -> &[TaskContext] {
        &self.context
    }

    /// Get error message (if failed)
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// Get the task configuration
    pub fn config(&self) -> &TaskConfig {
        &self.config
    }

    /// Check if task is ready to execute (all dependencies met)
    pub fn is_ready(&self, completed_tasks: &HashMap<String, TaskOutput>) -> bool {
        self.config
            .dependencies
            .iter()
            .all(|dep| completed_tasks.contains_key(dep))
    }

    /// Add context from a completed task
    pub fn add_context(&mut self, task_id: String, output: String, success: bool) {
        self.context.push(TaskContext {
            source_task_id: task_id,
            output,
            success,
            completed_at: chrono::Utc::now(),
        });
    }

    /// Mark task as in progress
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.started_at = Some(chrono::Utc::now());
        self.attempts += 1;
    }

    /// Mark task as completed
    pub fn complete(&mut self, output: TaskOutput) {
        self.status = TaskStatus::Completed;
        self.output = Some(output);
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark task as failed
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = TaskStatus::Failed;
        self.error = Some(error.into());
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Mark task as skipped
    pub fn skip(&mut self) {
        self.status = TaskStatus::Skipped;
        self.completed_at = Some(chrono::Utc::now());
    }

    /// Check if task can be retried
    pub fn can_retry(&self) -> bool {
        self.attempts <= self.config.max_retries
    }

    /// Reset task for retry
    pub fn reset(&mut self) {
        self.status = TaskStatus::Pending;
        self.output = None;
        self.error = None;
        self.started_at = None;
        self.completed_at = None;
    }

    /// Get execution duration in milliseconds
    pub fn execution_time_ms(&self) -> Option<u64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end - start).num_milliseconds() as u64),
            _ => None,
        }
    }

    /// Build the full prompt for this task including context
    pub fn build_prompt(&self) -> String {
        let mut prompt = format!(
            "# Task\n{}\n\n# Expected Output\n{}",
            self.config.description, self.config.expected_output
        );

        if !self.context.is_empty() {
            prompt.push_str("\n\n# Context from Previous Tasks\n");
            for ctx in &self.context {
                prompt.push_str(&format!(
                    "\n## From Task: {}\n{}\n",
                    ctx.source_task_id, ctx.output
                ));
            }
        }

        if let Some(instructions) = &self.config.context_instructions {
            prompt.push_str(&format!("\n\n# Additional Instructions\n{}", instructions));
        }

        prompt
    }
}

/// Builder for creating tasks with a fluent API
pub struct TaskBuilder {
    config: TaskConfig,
}

impl TaskBuilder {
    /// Create a new task builder
    pub fn new() -> Self {
        Self {
            config: TaskConfig::default(),
        }
    }

    /// Set the task ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.config.id = id.into();
        self
    }

    /// Set the task name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = Some(name.into());
        self
    }

    /// Set the task description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.description = description.into();
        self
    }

    /// Set the expected output
    pub fn expected_output(mut self, expected: impl Into<String>) -> Self {
        self.config.expected_output = expected.into();
        self
    }

    /// Assign an agent to this task
    pub fn agent(mut self, agent_id: impl Into<String>) -> Self {
        self.config.agent_id = Some(agent_id.into());
        self
    }

    /// Add a task dependency
    pub fn depends_on(mut self, task_id: impl Into<String>) -> Self {
        self.config.dependencies.push(task_id.into());
        self
    }

    /// Add multiple task dependencies
    pub fn depends_on_many(mut self, task_ids: &[impl AsRef<str>]) -> Self {
        for id in task_ids {
            self.config.dependencies.push(id.as_ref().to_string());
        }
        self
    }

    /// Set task as asynchronous
    pub fn is_async(mut self, is_async: bool) -> Self {
        self.config.is_async = is_async;
        self
    }

    /// Set timeout in seconds
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.timeout = Some(seconds);
        self
    }

    /// Enable human input for validation
    pub fn human_input(mut self, enabled: bool) -> Self {
        self.config.human_input = enabled;
        self
    }

    /// Set output file path
    pub fn output_file(mut self, path: impl Into<String>) -> Self {
        self.config.output_file = Some(path.into());
        self
    }

    /// Add a tool requirement
    pub fn tool(mut self, tool_name: impl Into<String>) -> Self {
        self.config.tools.push(tool_name.into());
        self
    }

    /// Add context instructions
    pub fn context_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.config.context_instructions = Some(instructions.into());
        self
    }

    /// Set whether to include in final output
    pub fn include_in_output(mut self, include: bool) -> Self {
        self.config.include_in_output = include;
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, retries: usize) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.metadata.insert(key.into(), value);
        self
    }

    /// Build the task
    pub fn build(self) -> Task {
        Task::new(self.config)
    }
}

impl Default for TaskBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_builder() {
        let task = Task::builder()
            .id("test-task")
            .name("Test Task")
            .description("This is a test task")
            .expected_output("Expected output here")
            .agent("agent-1")
            .build();

        assert_eq!(task.id(), "test-task");
        assert_eq!(task.name(), Some("Test Task"));
        assert_eq!(task.description(), "This is a test task");
        assert_eq!(task.expected_output(), "Expected output here");
        assert_eq!(task.agent_id(), Some("agent-1"));
    }

    #[test]
    fn test_task_dependencies() {
        let task = Task::builder()
            .id("task-3")
            .description("Depends on task 1 and 2")
            .expected_output("Combined output")
            .depends_on("task-1")
            .depends_on("task-2")
            .build();

        assert_eq!(task.dependencies(), &["task-1", "task-2"]);
    }

    #[test]
    fn test_task_lifecycle() {
        let mut task = Task::builder()
            .id("lifecycle-task")
            .description("Test lifecycle")
            .expected_output("Output")
            .build();

        assert_eq!(task.status(), TaskStatus::Pending);

        task.start();
        assert_eq!(task.status(), TaskStatus::InProgress);
        assert!(task.started_at.is_some());

        task.complete(TaskOutput::new("Done!"));
        assert_eq!(task.status(), TaskStatus::Completed);
        assert!(task.completed_at.is_some());
        assert!(task.output().is_some());
    }

    #[test]
    fn test_task_prompt_building() {
        let mut task = Task::builder()
            .description("Analyze the data")
            .expected_output("A detailed analysis report")
            .context_instructions("Focus on trends")
            .build();

        task.add_context("prev-task".to_string(), "Previous output".to_string(), true);

        let prompt = task.build_prompt();
        assert!(prompt.contains("Analyze the data"));
        assert!(prompt.contains("detailed analysis report"));
        assert!(prompt.contains("Previous output"));
        assert!(prompt.contains("Focus on trends"));
    }
}
