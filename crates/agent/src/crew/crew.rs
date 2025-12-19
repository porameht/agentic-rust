//! Crew Orchestrator
//!
//! A Crew is a team of agents working together to complete a set of tasks.
//! The crew manages agent coordination, task execution, and result aggregation.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::agent::{Agent, AgentError, AgentExecutor, ExecutionContext};
use super::config::{AgentsConfig, CrewYamlConfig, TasksConfig};
use super::memory::{CrewMemory, MemoryConfig};
use super::process::{Process, ProcessConfig};
use super::task::{Task, TaskError, TaskOutput};

/// Errors that can occur during crew operations
#[derive(Error, Debug)]
pub enum CrewError {
    #[error("Crew execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Agent error: {0}")]
    AgentError(#[from] AgentError),

    #[error("Task error: {0}")]
    TaskError(#[from] TaskError),

    #[error("No agents in crew")]
    NoAgents,

    #[error("No tasks in crew")]
    NoTasks,

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Circular dependency detected in tasks")]
    CircularDependency,

    #[error("Crew timeout after {0} seconds")]
    Timeout(u64),

    #[error("Manager agent required for hierarchical process")]
    ManagerRequired,

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

/// Configuration for a crew
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewConfig {
    /// Unique identifier for the crew
    pub id: String,

    /// Human-readable name
    pub name: Option<String>,

    /// Description of the crew's purpose
    pub description: Option<String>,

    /// Process configuration
    pub process: ProcessConfig,

    /// Memory configuration
    pub memory: Option<MemoryConfig>,

    /// Whether to enable verbose logging
    pub verbose: bool,

    /// Maximum crew execution time in seconds
    pub timeout: Option<u64>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for CrewConfig {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: None,
            description: None,
            process: ProcessConfig::default(),
            memory: None,
            verbose: false,
            timeout: None,
            metadata: HashMap::new(),
        }
    }
}

/// Result of crew execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewResult {
    /// Unique ID for this execution
    pub execution_id: String,

    /// Final combined output
    pub output: String,

    /// Individual task outputs
    pub task_outputs: HashMap<String, TaskOutput>,

    /// Execution statistics
    pub stats: ExecutionStats,

    /// Whether execution was successful
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Raw outputs from all tasks (in order)
    pub raw_outputs: Vec<String>,
}

/// Execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionStats {
    /// Total execution time in milliseconds
    pub total_time_ms: u64,

    /// Number of tasks executed
    pub tasks_executed: usize,

    /// Number of tasks that succeeded
    pub tasks_succeeded: usize,

    /// Number of tasks that failed
    pub tasks_failed: usize,

    /// Number of tasks skipped
    pub tasks_skipped: usize,

    /// Total LLM calls made
    pub total_llm_calls: usize,

    /// Total tokens used (if available)
    pub total_tokens: Option<usize>,

    /// Timestamp when execution started
    pub started_at: chrono::DateTime<chrono::Utc>,

    /// Timestamp when execution completed
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

/// Event listener for crew execution
#[async_trait]
pub trait CrewEventListener: Send + Sync {
    /// Called when crew execution starts
    async fn on_crew_start(&self, _crew_id: &str) {}

    /// Called when crew execution completes
    async fn on_crew_complete(&self, _crew_id: &str, _result: &CrewResult) {}

    /// Called when a task starts
    async fn on_task_start(&self, _task_id: &str, _agent_id: &str) {}

    /// Called when a task completes
    async fn on_task_complete(&self, _task_id: &str, _output: &TaskOutput) {}

    /// Called when a task fails
    async fn on_task_fail(&self, _task_id: &str, _error: &str) {}

    /// Called when an agent makes a tool call
    async fn on_tool_call(&self, _agent_id: &str, _tool_name: &str, _args: &serde_json::Value) {}
}

/// A crew of agents working together
pub struct Crew {
    /// Crew configuration
    config: CrewConfig,

    /// Agents in the crew
    agents: HashMap<String, Arc<Agent>>,

    /// Tasks to execute
    tasks: Vec<Task>,

    /// Crew memory
    memory: Option<CrewMemory>,

    /// Event listeners
    listeners: Vec<Arc<dyn CrewEventListener>>,

    /// Completed task outputs (for context passing)
    completed_outputs: RwLock<HashMap<String, TaskOutput>>,
}

impl Crew {
    /// Create a new crew builder
    pub fn builder() -> CrewBuilder {
        CrewBuilder::new()
    }

    /// Create a new crew with configuration
    pub fn new(config: CrewConfig) -> Self {
        let memory = config.memory.as_ref().map(|mc| CrewMemory::new(mc.clone()));

        Self {
            config,
            agents: HashMap::new(),
            tasks: Vec::new(),
            memory,
            listeners: Vec::new(),
            completed_outputs: RwLock::new(HashMap::new()),
        }
    }

    /// Get crew ID
    pub fn id(&self) -> &str {
        &self.config.id
    }

    /// Get crew name
    pub fn name(&self) -> Option<&str> {
        self.config.name.as_deref()
    }

    /// Add an agent to the crew
    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.insert(agent.id().to_string(), Arc::new(agent));
    }

    /// Add a task to the crew
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Add an event listener
    pub fn add_listener(&mut self, listener: Arc<dyn CrewEventListener>) {
        self.listeners.push(listener);
    }

    /// Validate the crew configuration
    pub fn validate(&self) -> Result<(), CrewError> {
        if self.agents.is_empty() {
            return Err(CrewError::NoAgents);
        }

        if self.tasks.is_empty() {
            return Err(CrewError::NoTasks);
        }

        // Verify all task agents exist
        for task in &self.tasks {
            if let Some(agent_id) = task.agent_id() {
                if !self.agents.contains_key(agent_id) {
                    return Err(CrewError::AgentNotFound(agent_id.to_string()));
                }
            }
        }

        // Check for circular dependencies
        if self.has_circular_dependencies() {
            return Err(CrewError::CircularDependency);
        }

        // Validate hierarchical process has manager
        if self.config.process.process_type == Process::Hierarchical
            && self.config.process.manager_model.is_none()
        {
            return Err(CrewError::ManagerRequired);
        }

        Ok(())
    }

    /// Check for circular dependencies in tasks
    fn has_circular_dependencies(&self) -> bool {
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for task in &self.tasks {
            if self.has_cycle(task.id(), &mut visited, &mut rec_stack) {
                return true;
            }
        }

        false
    }

    fn has_cycle(
        &self,
        task_id: &str,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> bool {
        if *rec_stack.get(task_id).unwrap_or(&false) {
            return true;
        }
        if *visited.get(task_id).unwrap_or(&false) {
            return false;
        }

        visited.insert(task_id.to_string(), true);
        rec_stack.insert(task_id.to_string(), true);

        if let Some(task) = self.tasks.iter().find(|t| t.id() == task_id) {
            for dep in task.dependencies() {
                if self.has_cycle(dep, visited, rec_stack) {
                    return true;
                }
            }
        }

        rec_stack.insert(task_id.to_string(), false);
        false
    }

    /// Execute the crew (kickoff)
    pub async fn kickoff(&mut self) -> Result<CrewResult, CrewError> {
        self.validate()?;

        let execution_id = Uuid::new_v4().to_string();
        let started_at = chrono::Utc::now();

        info!(
            crew_id = %self.config.id,
            execution_id = %execution_id,
            "Starting crew execution"
        );

        // Notify listeners
        for listener in &self.listeners {
            listener.on_crew_start(&self.config.id).await;
        }

        // Clear previous outputs
        {
            let mut outputs = self.completed_outputs.write().await;
            outputs.clear();
        }

        // Execute based on process type
        let result = match self.config.process.process_type {
            Process::Sequential => self.execute_sequential().await,
            Process::Hierarchical => self.execute_hierarchical().await,
            Process::Parallel => self.execute_parallel().await,
            Process::Custom => Err(CrewError::ExecutionFailed(
                "Custom process not implemented".to_string(),
            )),
        };

        let completed_at = chrono::Utc::now();
        let total_time_ms = (completed_at - started_at).num_milliseconds() as u64;

        // Build result
        let crew_result = match result {
            Ok(outputs) => {
                let raw_outputs: Vec<String> = self
                    .tasks
                    .iter()
                    .filter_map(|t| outputs.get(t.id()).map(|o| o.result.clone()))
                    .collect();

                let final_output = raw_outputs.join("\n\n---\n\n");

                let stats = ExecutionStats {
                    total_time_ms,
                    tasks_executed: self.tasks.len(),
                    tasks_succeeded: outputs.len(),
                    tasks_failed: self.tasks.len() - outputs.len(),
                    tasks_skipped: 0,
                    total_llm_calls: outputs.len(), // Placeholder
                    total_tokens: None,
                    started_at,
                    completed_at,
                };

                CrewResult {
                    execution_id,
                    output: final_output,
                    task_outputs: outputs,
                    stats,
                    success: true,
                    error: None,
                    raw_outputs,
                }
            }
            Err(e) => {
                let outputs = self.completed_outputs.read().await.clone();

                CrewResult {
                    execution_id,
                    output: String::new(),
                    task_outputs: outputs,
                    stats: ExecutionStats {
                        total_time_ms,
                        tasks_executed: 0,
                        tasks_succeeded: 0,
                        tasks_failed: 1,
                        tasks_skipped: 0,
                        total_llm_calls: 0,
                        total_tokens: None,
                        started_at,
                        completed_at,
                    },
                    success: false,
                    error: Some(e.to_string()),
                    raw_outputs: Vec::new(),
                }
            }
        };

        // Notify listeners
        for listener in &self.listeners {
            listener.on_crew_complete(&self.config.id, &crew_result).await;
        }

        info!(
            crew_id = %self.config.id,
            success = crew_result.success,
            total_time_ms = crew_result.stats.total_time_ms,
            "Crew execution completed"
        );

        Ok(crew_result)
    }

    /// Execute tasks sequentially
    async fn execute_sequential(&mut self) -> Result<HashMap<String, TaskOutput>, CrewError> {
        let mut outputs: HashMap<String, TaskOutput> = HashMap::new();

        for task in &mut self.tasks {
            // Check if dependencies are met
            let deps_met = task
                .dependencies()
                .iter()
                .all(|dep| outputs.contains_key(dep));

            if !deps_met {
                warn!(task_id = %task.id(), "Task dependencies not met, skipping");
                task.skip();
                continue;
            }

            // Add context from dependencies
            let deps: Vec<String> = task.dependencies().to_vec();
            for dep_id in deps {
                if let Some(dep_output) = outputs.get(&dep_id) {
                    task.add_context(dep_id, dep_output.result.clone(), true);
                }
            }

            // Get assigned agent
            let agent_id = task.agent_id().ok_or_else(|| {
                CrewError::ValidationFailed(format!("No agent assigned to task {}", task.id()))
            })?;

            let agent = self
                .agents
                .get(agent_id)
                .ok_or_else(|| CrewError::AgentNotFound(agent_id.to_string()))?
                .clone();

            // Notify listeners
            for listener in &self.listeners {
                listener.on_task_start(task.id(), agent_id).await;
            }

            debug!(task_id = %task.id(), agent_id = %agent_id, "Executing task");
            task.start();

            // Build execution context
            let context = ExecutionContext {
                task_description: task.description().to_string(),
                expected_output: task.expected_output().to_string(),
                context: task.context().iter().map(|c| c.output.clone()).collect(),
                available_tools: agent.tool_definitions(),
                shared_state: HashMap::new(),
                iteration: 0,
                max_iterations: agent.config().max_iterations,
            };

            // Execute the agent
            match agent.execute(context).await {
                Ok(result) => {
                    let output = TaskOutput::new(&result.output);
                    task.complete(output.clone());

                    // Notify listeners
                    for listener in &self.listeners {
                        listener.on_task_complete(task.id(), &output).await;
                    }

                    outputs.insert(task.id().to_string(), output);

                    info!(task_id = %task.id(), "Task completed successfully");
                }
                Err(e) => {
                    error!(task_id = %task.id(), error = %e, "Task execution failed");
                    task.fail(&e.to_string());

                    // Notify listeners
                    for listener in &self.listeners {
                        listener.on_task_fail(task.id(), &e.to_string()).await;
                    }

                    if self.config.process.fail_fast {
                        return Err(CrewError::ExecutionFailed(format!(
                            "Task {} failed: {}",
                            task.id(),
                            e
                        )));
                    }
                }
            }
        }

        // Update completed outputs
        {
            let mut completed = self.completed_outputs.write().await;
            *completed = outputs.clone();
        }

        Ok(outputs)
    }

    /// Execute tasks with hierarchical delegation
    async fn execute_hierarchical(&mut self) -> Result<HashMap<String, TaskOutput>, CrewError> {
        // In hierarchical mode, a manager agent decides task assignment
        // For now, fall back to sequential with logging
        warn!("Hierarchical process using sequential fallback");
        self.execute_sequential().await
    }

    /// Execute tasks in parallel where dependencies allow
    async fn execute_parallel(&mut self) -> Result<HashMap<String, TaskOutput>, CrewError> {
        // For now, fall back to sequential
        // TODO: Implement true parallel execution with dependency resolution
        warn!("Parallel process using sequential fallback");
        self.execute_sequential().await
    }

    /// Get crew configuration
    pub fn config(&self) -> &CrewConfig {
        &self.config
    }

    /// Get all agents
    pub fn agents(&self) -> &HashMap<String, Arc<Agent>> {
        &self.agents
    }

    /// Get all tasks
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
}

/// Builder for creating crews
pub struct CrewBuilder {
    config: CrewConfig,
    agents: Vec<Agent>,
    tasks: Vec<Task>,
    listeners: Vec<Arc<dyn CrewEventListener>>,
}

impl CrewBuilder {
    /// Create a new crew builder
    pub fn new() -> Self {
        Self {
            config: CrewConfig::default(),
            agents: Vec::new(),
            tasks: Vec::new(),
            listeners: Vec::new(),
        }
    }

    /// Set crew ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.config.id = id.into();
        self
    }

    /// Set crew name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = Some(name.into());
        self
    }

    /// Set crew description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.config.description = Some(description.into());
        self
    }

    /// Set process type
    pub fn process(mut self, process: Process) -> Self {
        self.config.process.process_type = process;
        self
    }

    /// Set process configuration
    pub fn process_config(mut self, config: ProcessConfig) -> Self {
        self.config.process = config;
        self
    }

    /// Set memory configuration
    pub fn memory(mut self, config: MemoryConfig) -> Self {
        self.config.memory = Some(config);
        self
    }

    /// Enable verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.config.verbose = verbose;
        self
    }

    /// Set timeout in seconds
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.config.timeout = Some(seconds);
        self
    }

    /// Add an agent
    pub fn agent(mut self, agent: Agent) -> Self {
        self.agents.push(agent);
        self
    }

    /// Add multiple agents
    pub fn agents(mut self, agents: impl IntoIterator<Item = Agent>) -> Self {
        self.agents.extend(agents);
        self
    }

    /// Add a task
    pub fn task(mut self, task: Task) -> Self {
        self.tasks.push(task);
        self
    }

    /// Add multiple tasks
    pub fn tasks(mut self, tasks: impl IntoIterator<Item = Task>) -> Self {
        self.tasks.extend(tasks);
        self
    }

    /// Add an event listener
    pub fn listener(mut self, listener: Arc<dyn CrewEventListener>) -> Self {
        self.listeners.push(listener);
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.metadata.insert(key.into(), value);
        self
    }

    /// Build the crew
    pub fn build(self) -> Crew {
        let mut crew = Crew::new(self.config);

        for agent in self.agents {
            crew.add_agent(agent);
        }

        for task in self.tasks {
            crew.add_task(task);
        }

        for listener in self.listeners {
            crew.add_listener(listener);
        }

        crew
    }
}

impl Default for CrewBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// YAML CONFIGURATION LOADER
// ============================================================================

/// Load and build crews from YAML configuration files
///
/// # Example
///
/// ```rust,ignore
/// let loader = CrewLoader::from_yaml(agents_yaml, tasks_yaml)?
///     .var("topic", "AI Agents")
///     .var("year", "2024");
///
/// let mut crew = loader.build("my_crew", Process::Sequential);
/// let result = crew.kickoff().await?;
/// ```
pub struct CrewLoader {
    agents: AgentsConfig,
    tasks: TasksConfig,
    vars: HashMap<String, String>,
}

impl CrewLoader {
    /// Create from YAML strings
    pub fn from_yaml(agents_yaml: &str, tasks_yaml: &str) -> Result<Self, super::config::ConfigError> {
        Ok(Self {
            agents: AgentsConfig::from_yaml(agents_yaml)?,
            tasks: TasksConfig::from_yaml(tasks_yaml)?,
            vars: HashMap::new(),
        })
    }

    /// Create from config directory (loads agents.yaml and tasks.yaml)
    pub fn from_dir<P: AsRef<std::path::Path>>(dir: P) -> Result<Self, super::config::ConfigError> {
        let config = CrewYamlConfig::from_directory(dir)?;
        Ok(Self {
            agents: config.agents,
            tasks: config.tasks,
            vars: HashMap::new(),
        })
    }

    /// Add a template variable (chainable)
    pub fn var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.vars.insert(key.into(), value.into());
        self
    }

    /// Get agent by ID (with variable substitution)
    pub fn agent(&self, id: &str) -> Option<Agent> {
        let mut cfg = self.agents.agents.get(id)?.clone();
        cfg.role = super::config::substitute_variables(&cfg.role, &self.vars);
        cfg.goal = super::config::substitute_variables(&cfg.goal, &self.vars);
        cfg.backstory = super::config::substitute_variables(&cfg.backstory, &self.vars);
        Some(cfg.to_agent(id))
    }

    /// Get task by ID (with variable substitution)
    pub fn task(&self, id: &str) -> Option<Task> {
        let mut cfg = self.tasks.tasks.get(id)?.clone();
        cfg.description = super::config::substitute_variables(&cfg.description, &self.vars);
        cfg.expected_output = super::config::substitute_variables(&cfg.expected_output, &self.vars);
        Some(cfg.to_task(id))
    }

    /// Build crew with specified agents and tasks
    pub fn build(&self, name: &str, agent_ids: &[&str], task_ids: &[&str], process: Process) -> Crew {
        let mut builder = Crew::builder().id(name).name(name).process(process);

        for id in agent_ids {
            if let Some(agent) = self.agent(id) {
                builder = builder.agent(agent);
            }
        }

        for id in task_ids {
            if let Some(task) = self.task(id) {
                builder = builder.task(task);
            }
        }

        builder.build()
    }

    /// Build crew with all agents and tasks from config
    pub fn build_all(&self, name: &str, process: Process) -> Crew {
        let agent_ids: Vec<&str> = self.agents.agents.keys().map(|s| s.as_str()).collect();
        let task_ids: Vec<&str> = self.tasks.tasks.keys().map(|s| s.as_str()).collect();
        self.build(name, &agent_ids, &task_ids, process)
    }
}

impl CrewLoader {
    /// Load example configuration (for demos/testing)
    pub fn example() -> Result<Self, super::config::ConfigError> {
        Self::from_yaml(
            super::config::example_agents_yaml(),
            super::config::example_tasks_yaml(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crew_builder() {
        let agent = Agent::builder()
            .id("test-agent")
            .role("Tester")
            .goal("Test things")
            .backstory("Expert tester")
            .build();

        let task = Task::builder()
            .id("test-task")
            .description("Test the system")
            .expected_output("Test results")
            .agent("test-agent")
            .build();

        let crew = Crew::builder()
            .id("test-crew")
            .name("Test Crew")
            .agent(agent)
            .task(task)
            .build();

        assert_eq!(crew.id(), "test-crew");
        assert_eq!(crew.agents().len(), 1);
        assert_eq!(crew.tasks().len(), 1);
    }

    #[test]
    fn test_crew_validation() {
        let crew = Crew::builder().id("empty-crew").build();

        assert!(matches!(crew.validate(), Err(CrewError::NoAgents)));

        let agent = Agent::builder()
            .id("agent")
            .role("Role")
            .goal("Goal")
            .backstory("Story")
            .build();

        let crew = Crew::builder().agent(agent).build();

        assert!(matches!(crew.validate(), Err(CrewError::NoTasks)));
    }

    #[tokio::test]
    async fn test_sequential_execution() {
        let agent = Agent::builder()
            .id("worker")
            .role("Worker")
            .goal("Complete tasks")
            .backstory("Dedicated worker")
            .build();

        let task = Task::builder()
            .id("task-1")
            .description("Do work")
            .expected_output("Work done")
            .agent("worker")
            .build();

        let mut crew = Crew::builder().agent(agent).task(task).build();

        let result = crew.kickoff().await.unwrap();
        assert!(result.success);
        assert_eq!(result.task_outputs.len(), 1);
    }

    #[test]
    fn test_crew_loader() {
        let loader = CrewLoader::example().unwrap().var("topic", "AI Agents");

        let researcher = loader.agent("researcher").unwrap();
        assert_eq!(researcher.role(), "Senior Data Researcher");
        assert!(researcher.goal().contains("AI Agents"));

        let task = loader.task("research_task").unwrap();
        assert!(task.description().contains("AI Agents"));
    }

    #[test]
    fn test_crew_loader_build() {
        let loader = CrewLoader::example().unwrap().var("topic", "ML");

        let crew = loader.build(
            "test",
            &["researcher", "reporting_analyst"],
            &["research_task", "reporting_task"],
            Process::Sequential,
        );

        assert_eq!(crew.agents().len(), 2);
        assert_eq!(crew.tasks().len(), 2);
    }

    #[tokio::test]
    async fn test_crew_loader_execution() {
        let loader = CrewLoader::example().unwrap().var("topic", "Rust");

        let mut crew = loader.build(
            "test",
            &["researcher"],
            &["research_task"],
            Process::Sequential,
        );

        assert!(crew.kickoff().await.unwrap().success);
    }
}
