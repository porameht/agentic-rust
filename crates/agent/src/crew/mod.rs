//! CrewAI-style Multi-Agent Orchestration Framework
//!
//! This module implements a CrewAI-inspired architecture for orchestrating
//! multiple AI agents to work together on complex tasks.
//!
//! # Core Concepts
//!
//! - **Agent**: Autonomous units with roles, goals, backstories, and tools
//! - **Task**: Specific assignments with expected outputs and agent assignments
//! - **Crew**: Teams of agents that collaborate to complete tasks
//! - **Process**: How tasks are executed (Sequential, Hierarchical)
//! - **Flow**: Event-driven workflows for complex orchestration
//!
//! # Example
//!
//! ```rust,ignore
//! use agent::crew::{Agent, Task, Crew, Process};
//!
//! // Create agents with specific roles
//! let researcher = Agent::builder()
//!     .role("Senior Research Analyst")
//!     .goal("Conduct thorough research and provide accurate insights")
//!     .backstory("Expert researcher with 10 years of experience in data analysis")
//!     .model("gpt-4")
//!     .build();
//!
//! let writer = Agent::builder()
//!     .role("Content Writer")
//!     .goal("Create compelling content based on research")
//!     .backstory("Award-winning writer specializing in technical content")
//!     .model("gpt-4")
//!     .build();
//!
//! // Create tasks
//! let research_task = Task::builder()
//!     .description("Research AI agent frameworks and their architectures")
//!     .expected_output("Comprehensive report with key findings")
//!     .agent(&researcher)
//!     .build();
//!
//! let writing_task = Task::builder()
//!     .description("Write a blog post based on the research findings")
//!     .expected_output("Engaging blog post of 1000 words")
//!     .agent(&writer)
//!     .context(&[&research_task])
//!     .build();
//!
//! // Create and run the crew
//! let crew = Crew::builder()
//!     .agents(&[researcher, writer])
//!     .tasks(&[research_task, writing_task])
//!     .process(Process::Sequential)
//!     .build();
//!
//! let result = crew.kickoff().await?;
//! ```

pub mod agent;
pub mod config;
pub mod crew;
pub mod examples;
pub mod flow;
pub mod integration;
pub mod memory;
pub mod process;
pub mod prompts;
pub mod task;
pub mod tools;

#[cfg(test)]
mod tests;

pub use agent::{Agent, AgentBuilder as CrewAgentBuilder, AgentConfig as CrewAgentConfig};
pub use crew::{Crew, CrewBuilder, CrewConfig, CrewLoader, CrewResult};
pub use flow::{Flow, FlowBuilder, FlowState, StateTransition, TransitionCondition};
pub use memory::{Memory, MemoryConfig, MemoryType};
pub use process::{Process, ProcessConfig};
pub use task::{Task, TaskBuilder, TaskConfig, TaskContext, TaskOutput, TaskStatus};

// Re-export example crews
pub use examples::{
    create_code_review_crew, create_content_flow, create_research_crew, create_sales_crew,
    create_support_flow,
};

// Re-export integration examples
pub use integration::{
    create_content_pipeline, run_content_pipeline, run_simple_crew_example, ContentPipeline,
    PipelineResult, PipelineStats,
};

// Re-export prompt configuration
pub use prompts::{
    crew_prompts, AgentPromptTemplates, CrewPromptConfig, CrewPromptTemplates, I18nPrompts,
    PromptBuilder, RolePromptConfig, RolePrompts, TaskPromptTemplates,
};

// Re-export YAML configuration
pub use config::{
    example_agents_yaml, example_tasks_yaml, substitute_variables, AgentYamlConfig,
    AgentsConfig, ConfigError, CrewYamlConfig, TaskYamlConfig, TasksConfig,
};

// Re-export tools
pub use tools::{
    BaseTool, CrewToolDefinition, DynamicTool, FileReadInput, FileReadTool, FileWriteInput,
    FileWriteTool, ReplInput, ReplTool, ToolError, ToolInput, ToolRegistry, WebSearchInput,
    WebSearchTool,
};
