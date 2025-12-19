//! YAML Configuration Loading for CrewAI-style Agents and Tasks
//!
//! This module provides configuration loading from YAML files matching the
//! CrewAI Python patterns (agents.yaml, tasks.yaml).
//!
//! # Example agents.yaml
//!
//! ```yaml
//! researcher:
//!   role: Senior Data Researcher
//!   goal: Discover emerging trends and insights about {topic}
//!   backstory: >
//!     You're a seasoned researcher with expertise in finding
//!     the most relevant information and presenting it clearly.
//!
//! reporting_analyst:
//!   role: Reporting Analyst
//!   goal: Create detailed reports based on analysis
//!   backstory: >
//!     You're a meticulous analyst with a talent for turning
//!     complex data into clear and concise reports.
//! ```
//!
//! # Example tasks.yaml
//!
//! ```yaml
//! research_task:
//!   description: >
//!     Conduct thorough research about {topic}.
//!     Make sure to find the most relevant information.
//!   expected_output: >
//!     A list with 10 bullet points of the most relevant
//!     information about {topic}.
//!   agent: researcher
//!
//! reporting_task:
//!   description: >
//!     Review the context and create a detailed report.
//!   expected_output: >
//!     A fully fledged report with main topics, each with
//!     a full section of information.
//!   agent: reporting_analyst
//!   output_file: report.md
//!   context:
//!     - research_task
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// YAML configuration for an agent (matching CrewAI Python pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentYamlConfig {
    /// The role of the agent (e.g., "Senior Data Researcher")
    pub role: String,

    /// The primary goal the agent is trying to achieve
    /// Supports template variables like {topic}
    pub goal: String,

    /// Background story providing personality and context
    pub backstory: String,

    /// LLM model to use (optional, defaults to crew default)
    #[serde(default)]
    pub llm: Option<String>,

    /// Tools available to this agent
    #[serde(default)]
    pub tools: Vec<String>,

    /// Whether the agent can delegate tasks
    #[serde(default)]
    pub allow_delegation: bool,

    /// Whether verbose logging is enabled
    #[serde(default)]
    pub verbose: bool,

    /// Maximum iterations for task execution
    #[serde(default)]
    pub max_iter: Option<usize>,

    /// Maximum execution time in seconds
    #[serde(default)]
    pub max_execution_time: Option<u64>,

    /// Maximum retry attempts
    #[serde(default)]
    pub max_retry_limit: Option<usize>,

    /// Whether to respect context window
    #[serde(default = "default_true")]
    pub respect_context_window: bool,

    /// Code execution mode
    #[serde(default)]
    pub code_execution_mode: Option<String>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

/// YAML configuration for a task (matching CrewAI Python pattern)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskYamlConfig {
    /// Detailed description of what the task should accomplish
    /// Supports template variables like {topic}, {current_year}
    pub description: String,

    /// Description of the expected output format and content
    pub expected_output: String,

    /// ID of the agent assigned to this task
    pub agent: String,

    /// Output file path (if task should write to file)
    #[serde(default)]
    pub output_file: Option<String>,

    /// Context from other tasks (task IDs)
    #[serde(default)]
    pub context: Vec<String>,

    /// Whether this task is asynchronous
    #[serde(default, rename = "async_execution")]
    pub is_async: bool,

    /// Whether to use human feedback for validation
    #[serde(default)]
    pub human_input: bool,

    /// Specific tools this task should use
    #[serde(default)]
    pub tools: Vec<String>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Collection of agents loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentsConfig {
    #[serde(flatten)]
    pub agents: HashMap<String, AgentYamlConfig>,
}

/// Collection of tasks loaded from YAML
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TasksConfig {
    #[serde(flatten)]
    pub tasks: HashMap<String, TaskYamlConfig>,
}

/// Combined crew configuration
#[derive(Debug, Clone, Default)]
pub struct CrewYamlConfig {
    pub agents: AgentsConfig,
    pub tasks: TasksConfig,
}

impl CrewYamlConfig {
    /// Load crew configuration from a directory containing agents.yaml and tasks.yaml
    pub fn from_directory<P: AsRef<Path>>(config_dir: P) -> Result<Self, ConfigError> {
        let config_dir = config_dir.as_ref();

        let agents_path = config_dir.join("agents.yaml");
        let tasks_path = config_dir.join("tasks.yaml");

        let agents = if agents_path.exists() {
            AgentsConfig::from_file(&agents_path)?
        } else {
            AgentsConfig::default()
        };

        let tasks = if tasks_path.exists() {
            TasksConfig::from_file(&tasks_path)?
        } else {
            TasksConfig::default()
        };

        Ok(Self { agents, tasks })
    }

    /// Load from separate files
    pub fn from_files<P: AsRef<Path>>(
        agents_file: Option<P>,
        tasks_file: Option<P>,
    ) -> Result<Self, ConfigError> {
        let agents = match agents_file {
            Some(path) => AgentsConfig::from_file(path)?,
            None => AgentsConfig::default(),
        };

        let tasks = match tasks_file {
            Some(path) => TasksConfig::from_file(path)?,
            None => TasksConfig::default(),
        };

        Ok(Self { agents, tasks })
    }

    /// Get an agent configuration by ID
    pub fn get_agent(&self, id: &str) -> Option<&AgentYamlConfig> {
        self.agents.agents.get(id)
    }

    /// Get a task configuration by ID
    pub fn get_task(&self, id: &str) -> Option<&TaskYamlConfig> {
        self.tasks.tasks.get(id)
    }

    /// Get all agent IDs
    pub fn agent_ids(&self) -> impl Iterator<Item = &String> {
        self.agents.agents.keys()
    }

    /// Get all task IDs
    pub fn task_ids(&self) -> impl Iterator<Item = &String> {
        self.tasks.tasks.keys()
    }

    /// Substitute template variables in all configurations
    pub fn with_variables(mut self, vars: &HashMap<String, String>) -> Self {
        // Substitute in agents
        for agent in self.agents.agents.values_mut() {
            agent.role = substitute_variables(&agent.role, vars);
            agent.goal = substitute_variables(&agent.goal, vars);
            agent.backstory = substitute_variables(&agent.backstory, vars);
        }

        // Substitute in tasks
        for task in self.tasks.tasks.values_mut() {
            task.description = substitute_variables(&task.description, vars);
            task.expected_output = substitute_variables(&task.expected_output, vars);
        }

        self
    }
}

impl AgentsConfig {
    /// Load agents from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ConfigError::IoError(format!("Failed to read agents file: {}", e))
        })?;
        Self::from_yaml(&content)
    }

    /// Parse agents from YAML string
    pub fn from_yaml(content: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse agents YAML: {}", e)))
    }
}

impl TasksConfig {
    /// Load tasks from a YAML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(format!("Failed to read tasks file: {}", e)))?;
        Self::from_yaml(&content)
    }

    /// Parse tasks from YAML string
    pub fn from_yaml(content: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(content)
            .map_err(|e| ConfigError::ParseError(format!("Failed to parse tasks YAML: {}", e)))
    }
}

/// Substitute template variables in a string
/// Variables are in the format {variable_name}
pub fn substitute_variables(template: &str, vars: &HashMap<String, String>) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

// ============================================================================
// BUILDER INTEGRATION
// ============================================================================

use super::agent::{Agent, AgentBuilder};
use super::task::{Task, TaskBuilder};

impl AgentYamlConfig {
    /// Convert to Agent using the builder
    pub fn to_agent(&self, id: &str) -> Agent {
        let mut builder = AgentBuilder::new()
            .id(id)
            .role(&self.role)
            .goal(&self.goal)
            .backstory(&self.backstory)
            .allow_delegation(self.allow_delegation)
            .verbose(self.verbose);

        if let Some(llm) = &self.llm {
            builder = builder.model(llm);
        }

        if let Some(max_iter) = self.max_iter {
            builder = builder.max_iterations(max_iter);
        }

        if let Some(max_time) = self.max_execution_time {
            builder = builder.max_execution_time(max_time);
        }

        for tool in &self.tools {
            builder = builder.tool_name(tool);
        }

        builder.build()
    }
}

impl TaskYamlConfig {
    /// Convert to Task using the builder
    pub fn to_task(&self, id: &str) -> Task {
        let mut builder = TaskBuilder::new()
            .id(id)
            .description(&self.description)
            .expected_output(&self.expected_output)
            .agent(&self.agent)
            .is_async(self.is_async)
            .human_input(self.human_input);

        if let Some(output_file) = &self.output_file {
            builder = builder.output_file(output_file);
        }

        for dep in &self.context {
            builder = builder.depends_on(dep);
        }

        for tool in &self.tools {
            builder = builder.tool(tool);
        }

        builder.build()
    }
}

// ============================================================================
// EXAMPLE CONFIGURATIONS
// ============================================================================

/// Create example agents configuration matching data-analyst-agent
pub fn example_agents_yaml() -> &'static str {
    r#"researcher:
  role: Senior Data Researcher
  goal: >
    Uncover cutting-edge developments in {topic}
  backstory: >
    You're a seasoned researcher with a knack for uncovering the latest
    developments in {topic}. Known for your ability to find the most relevant
    information and present it in a clear and concise manner.

reporting_analyst:
  role: Reporting Analyst
  goal: >
    Create detailed reports based on {topic} data analysis and research findings
  backstory: >
    You're a meticulous analyst with a keen eye for detail. You're known for
    your ability to turn complex data into clear and concise reports, making
    it easy for others to understand and act on the information you provide.

coding_agent:
  role: Python Developer
  goal: >
    Craft well-designed and thought-out code solutions for {problem}
  backstory: >
    You are a skilled Python developer with expertise in writing clean,
    efficient, and scalable code. You have a deep understanding of software
    engineering principles and best practices.

executing_agent:
  role: Python Code Executor
  goal: >
    Execute Python code and return the results for {problem}
  backstory: >
    You are a developer who can execute code, debug, and optimize Python
    solutions. You have access to a REPL tool for running code.
  tools:
    - repl
    - file_read
"#
}

/// Create example tasks configuration matching data-analyst-agent
pub fn example_tasks_yaml() -> &'static str {
    r#"research_task:
  description: >
    Conduct a thorough research about {topic}.
    Make sure you find any interesting and relevant information given
    the current year is {current_year}.
  expected_output: >
    A list with 10 bullet points of the most relevant information about {topic}.
  agent: researcher

reporting_task:
  description: >
    Review the context you got and expand each topic into a full section for a report.
    Make sure the report is detailed and contains any and all relevant information.
  expected_output: >
    A fully fledged report with the main topics, each with a full section of information.
    Formatted as markdown without '```'.
  agent: reporting_analyst
  output_file: report.md
  context:
    - research_task

coding_task:
  description: >
    Write Python code to solve the following problem: {problem}.
    Ensure the code is clean, efficient, and well-documented.
  expected_output: >
    A Python code solution with the output assigned to the 'result' variable.
  agent: coding_agent

executing_task:
  description: >
    Execute the Python code provided in the context and return the result.
    Debug and optimize if necessary.
  expected_output: >
    The result of executing the Python code for the problem.
  agent: executing_agent
  context:
    - coding_task
  tools:
    - repl
"#
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_agents_yaml() {
        let yaml = example_agents_yaml();
        let config = AgentsConfig::from_yaml(yaml).unwrap();

        assert!(config.agents.contains_key("researcher"));
        assert!(config.agents.contains_key("reporting_analyst"));
        assert!(config.agents.contains_key("coding_agent"));
        assert!(config.agents.contains_key("executing_agent"));

        let researcher = config.agents.get("researcher").unwrap();
        assert_eq!(researcher.role, "Senior Data Researcher");
        assert!(researcher.goal.contains("{topic}"));
    }

    #[test]
    fn test_parse_tasks_yaml() {
        let yaml = example_tasks_yaml();
        let config = TasksConfig::from_yaml(yaml).unwrap();

        assert!(config.tasks.contains_key("research_task"));
        assert!(config.tasks.contains_key("reporting_task"));
        assert!(config.tasks.contains_key("coding_task"));
        assert!(config.tasks.contains_key("executing_task"));

        let reporting = config.tasks.get("reporting_task").unwrap();
        assert_eq!(reporting.agent, "reporting_analyst");
        assert_eq!(reporting.output_file, Some("report.md".to_string()));
        assert_eq!(reporting.context, vec!["research_task"]);
    }

    #[test]
    fn test_substitute_variables() {
        let mut vars = HashMap::new();
        vars.insert("topic".to_string(), "AI Agents".to_string());
        vars.insert("current_year".to_string(), "2024".to_string());

        let template = "Research about {topic} in {current_year}";
        let result = substitute_variables(template, &vars);

        assert_eq!(result, "Research about AI Agents in 2024");
    }

    #[test]
    fn test_agent_to_builder() {
        let yaml = r#"
role: Test Role
goal: Test Goal
backstory: Test Backstory
verbose: true
allow_delegation: true
"#;
        let config: AgentYamlConfig = serde_yaml::from_str(yaml).unwrap();
        let agent = config.to_agent("test-agent");

        assert_eq!(agent.id(), "test-agent");
        assert_eq!(agent.role(), "Test Role");
        assert_eq!(agent.goal(), "Test Goal");
        assert_eq!(agent.backstory(), "Test Backstory");
    }

    #[test]
    fn test_task_to_builder() {
        let yaml = r#"
description: Test Description
expected_output: Test Output
agent: test-agent
output_file: output.md
context:
  - prev-task
"#;
        let config: TaskYamlConfig = serde_yaml::from_str(yaml).unwrap();
        let task = config.to_task("test-task");

        assert_eq!(task.id(), "test-task");
        assert_eq!(task.description(), "Test Description");
        assert_eq!(task.expected_output(), "Test Output");
        assert_eq!(task.agent_id(), Some("test-agent"));
        assert_eq!(task.dependencies(), &["prev-task"]);
    }

    #[test]
    fn test_config_with_variables() {
        let mut config = CrewYamlConfig::default();

        let agents_yaml = example_agents_yaml();
        config.agents = AgentsConfig::from_yaml(agents_yaml).unwrap();

        let tasks_yaml = example_tasks_yaml();
        config.tasks = TasksConfig::from_yaml(tasks_yaml).unwrap();

        let mut vars = HashMap::new();
        vars.insert("topic".to_string(), "Machine Learning".to_string());
        vars.insert("current_year".to_string(), "2024".to_string());
        vars.insert("problem".to_string(), "sorting algorithm".to_string());

        let config = config.with_variables(&vars);

        let researcher = config.agents.agents.get("researcher").unwrap();
        assert!(researcher.goal.contains("Machine Learning"));
        assert!(!researcher.goal.contains("{topic}"));

        let research_task = config.tasks.tasks.get("research_task").unwrap();
        assert!(research_task.description.contains("Machine Learning"));
        assert!(research_task.description.contains("2024"));
    }
}
