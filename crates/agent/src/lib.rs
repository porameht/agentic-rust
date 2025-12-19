//! LLM Agent Framework with Two Execution Flows
//!
//! This crate provides two distinct approaches for building AI agents:
//!
//! # Flow 1: Single Agent with RAG (rig-core based)
//!
//! Traditional single-agent approach with Retrieval-Augmented Generation.
//! Best for: Q&A systems, document search, knowledge bases.
//!
//! ```rust,ignore
//! use agent::{AgentBuilder, RagAgent};
//! use rag_core::Retriever;
//!
//! // Build agent configuration
//! let config = AgentBuilder::new("gpt-4")
//!     .preamble("You are a helpful assistant.")
//!     .temperature(0.7)
//!     .top_k_documents(5)
//!     .tool("search")
//!     .build();
//!
//! // Create RAG agent with retriever
//! let agent = RagAgent::new(config, retriever);
//! let response = agent.chat("What is Rust?").await?;
//! ```
//!
//! # Flow 2: Multi-Agent Orchestration (CrewAI-style)
//!
//! Multiple agents collaborating on complex tasks with role specialization.
//! Best for: Research pipelines, content creation, code review, data analysis.
//!
//! ```rust,ignore
//! use agent::crew::{Agent, Task, Crew, Process, CrewLoader};
//!
//! // Option A: Builder pattern
//! let researcher = Agent::builder()
//!     .role("Senior Researcher")
//!     .goal("Conduct thorough research")
//!     .build();
//!
//! let task = Task::builder()
//!     .description("Research AI frameworks")
//!     .expected_output("Comprehensive report")
//!     .build();
//!
//! let mut crew = Crew::builder()
//!     .agent(researcher)
//!     .task(task)
//!     .process(Process::Sequential)
//!     .build();
//!
//! let result = crew.kickoff().await?;
//!
//! // Option B: YAML configuration
//! let loader = CrewLoader::from_dir("config/")?
//!     .var("topic", "AI Agents");
//!
//! let mut crew = loader.build_all("research_crew", Process::Sequential);
//! let result = crew.kickoff().await?;
//! ```
//!
//! # Architecture Overview
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     agent crate                             │
//! ├─────────────────────────────┬───────────────────────────────┤
//! │   Flow 1: Single Agent      │   Flow 2: Multi-Agent (Crew)  │
//! │   ─────────────────────     │   ──────────────────────────  │
//! │   • AgentBuilder            │   • Agent, Task, Crew         │
//! │   • RagAgent                │   • Flow (state machine)      │
//! │   • SalesAgentBuilder       │   • CrewLoader (YAML config)  │
//! │   • tools/ (rig tools)      │   • ToolRegistry (crew tools) │
//! │                             │   • Memory, Process           │
//! ├─────────────────────────────┴───────────────────────────────┤
//! │                    Shared: rig-core, rag-core               │
//! └─────────────────────────────────────────────────────────────┘
//! ```

// ============================================================================
// FLOW 1: SINGLE AGENT WITH RAG (rig-core based)
// ============================================================================

pub mod builder;
pub mod prompts;
pub mod rag_agent;
pub mod sales_agent;
pub mod tools;

// Single-agent exports
pub use builder::AgentBuilder;
pub use rag_agent::RagAgent;
pub use sales_agent::SalesAgentBuilder;

// ============================================================================
// FLOW 2: MULTI-AGENT ORCHESTRATION (CrewAI-style)
// ============================================================================

pub mod crew;

// Crew module re-exports for convenience
// Core types
pub use crew::{
    Agent as CrewAgent, Crew, CrewBuilder, CrewLoader, CrewResult, Process, ProcessConfig,
};

// Task management
pub use crew::{Task, TaskBuilder, TaskOutput, TaskStatus};

// Flow orchestration (state machine for crews)
pub use crew::{Flow, FlowBuilder, FlowState, StateTransition, TransitionCondition};

// Memory system
pub use crew::{Memory, MemoryConfig, MemoryType};

// Configuration (YAML loading)
pub use crew::{
    example_agents_yaml, example_tasks_yaml, substitute_variables, AgentsConfig, ConfigError,
    CrewYamlConfig, TaskYamlConfig, TasksConfig,
};

// Tools (crew-specific)
pub use crew::{
    BaseTool, CrewToolDefinition, DynamicTool, FileReadTool, FileWriteTool, ReplTool, ToolError,
    ToolInput, ToolRegistry, WebSearchTool,
};

// Prompts
pub use crew::{
    crew_prompts, AgentPromptTemplates, CrewPromptConfig, CrewPromptTemplates, PromptBuilder,
    RolePromptConfig, RolePrompts,
};

// Examples
pub use crew::{
    create_code_review_crew, create_content_flow, create_content_pipeline, create_research_crew,
    create_sales_crew, create_support_flow, run_content_pipeline, run_simple_crew_example,
    ContentPipeline, PipelineResult, PipelineStats,
};
