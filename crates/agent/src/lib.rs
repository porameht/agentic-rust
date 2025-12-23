//! LLM Agent Framework with Two Execution Flows
//!
//! This crate provides two distinct approaches for building AI agents:
//!
//! # Flow 1: ReAct Agent with RAG (Queue-based, Scalable)
//!
//! ReAct (Reasoning + Acting) agent with queue-based processing for production scale.
//! Uses iterative reasoning loop: Think → Act → Observe → Repeat until done.
//! Best for: Complex Q&A, tool-using agents, research assistants.
//!
//! ```text
//! ┌──────────┐    ┌─────────┐    ┌────────┐    ┌─────────────────────────┐
//! │   API    │───▶│  Queue  │───▶│ Worker │───▶│     ReActAgent          │
//! │ Request  │    │ (Redis) │    │        │    │                         │
//! └──────────┘    └─────────┘    └────────┘    │  ┌─────────────────────┐│
//!                                              │  │ REASONING LOOP      ││
//!                                              │  │ ┌─────┐             ││
//!                                              │  │ │Think│→ Decide     ││
//!                                              │  │ └──┬──┘             ││
//!                                              │  │    ↓                ││
//!                                              │  │ ┌─────┐   ┌───────┐││
//!                                              │  │ │ Act │──▶│Observe│││
//!                                              │  │ └─────┘   └───┬───┘││
//!                                              │  │       ↑       │    ││
//!                                              │  │       └───────┘    ││
//!                                              │  └─────────────────────┘│
//!                                              └─────────────────────────┘
//! ```
//!
//! ```rust,ignore
//! use agent::{ReActAgent, ReActConfig};
//! use agent::tools::SearchTool;
//!
//! // Build ReAct agent with tools
//! let config = ReActConfig::builder()
//!     .model("gpt-4")
//!     .max_iterations(10)
//!     .temperature(0.3)
//!     .use_rag(true)
//!     .build();
//!
//! let mut agent = ReActAgent::new(config)
//!     .with_tool(SearchTool::new())
//!     .with_tool(CalculatorTool::new());
//!
//! // Run with reasoning trace
//! let response = agent.run("What is the population of Tokyo?").await?;
//! println!("Answer: {}", response.final_answer);
//! println!("Steps: {:?}", response.trace);
//! ```
//!
//! ## Legacy: Simple RAG Agent (without reasoning loop)
//!
//! ```rust,ignore
//! use agent::{AgentBuilder, RagAgent};
//! use rag_core::Retriever;
//!
//! let config = AgentBuilder::new("gpt-4")
//!     .preamble("You are a helpful assistant.")
//!     .temperature(0.7)
//!     .top_k_documents(5)
//!     .build();
//!
//! let agent = RagAgent::new(config, retriever);
//! let response = agent.chat("What is Rust?").await?;
//! ```
//!
//! # Flow 2: Multi-Agent Orchestration (CrewAI-style, Direct Execution)
//!
//! Multiple agents collaborating on complex tasks with role specialization.
//! Runs directly without queue - suitable for batch/background processing.
//! Best for: Research pipelines, content creation, code review, data analysis.
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                     Direct Execution                        │
//! │  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
//! │  │Researcher│───▶│  Writer  │───▶│  Editor  │              │
//! │  │  Agent   │    │  Agent   │    │  Agent   │              │
//! │  └──────────┘    └──────────┘    └──────────┘              │
//! │       │               │               │                     │
//! │       ▼               ▼               ▼                     │
//! │  ┌──────────┐    ┌──────────┐    ┌──────────┐              │
//! │  │  Task 1  │───▶│  Task 2  │───▶│  Task 3  │──▶ Result    │
//! │  └──────────┘    └──────────┘    └──────────┘              │
//! └─────────────────────────────────────────────────────────────┘
//! ```
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
//! ┌─────────────────────────────────────────────────────────────────────────┐
//! │                           agent crate                                   │
//! ├───────────────────────────────────┬─────────────────────────────────────┤
//! │   Flow 1: Single Agent + Queue    │   Flow 2: Multi-Agent (CrewAI)      │
//! │   ────────────────────────────    │   ─────────────────────────────     │
//! │                                   │                                     │
//! │   API Request                     │   Direct Execution                  │
//! │       ↓                           │       ↓                             │
//! │   Queue (Redis/etc)               │   Crew::builder()                   │
//! │       ↓                           │       .agent(researcher)            │
//! │   Worker                          │       .agent(writer)                │
//! │       ↓                           │       .task(...)                    │
//! │   RagAgent.chat()                 │       .build()                      │
//! │       ↓                           │       ↓                             │
//! │   Response                        │   crew.kickoff().await              │
//! │                                   │                                     │
//! │   Components:                     │   Components:                       │
//! │   • AgentBuilder                  │   • Agent, Task, Crew               │
//! │   • RagAgent                      │   • Flow (state machine)            │
//! │   • SalesAgentBuilder             │   • Process (Sequential/Hier.)      │
//! │   • tools/ (rig tools)            │   • Memory, ToolRegistry            │
//! │                                   │   • YAML Config (CrewLoader)        │
//! ├───────────────────────────────────┴─────────────────────────────────────┤
//! │                        Shared: rig-core, rag-core                       │
//! └─────────────────────────────────────────────────────────────────────────┘
//! ```

// ============================================================================
// FLOW 1: SINGLE AGENT WITH RAG (rig-core based)
// ============================================================================

pub mod builder;
pub mod prompts;
pub mod rag_agent;
pub mod react_agent;
pub mod rig_integration;
pub mod sales_agent;
pub mod tools;

// Single-agent exports
pub use builder::AgentBuilder;
pub use rag_agent::RagAgent;
pub use sales_agent::SalesAgentBuilder;

// ReAct agent exports (Flow 1 with reasoning loop)
pub use react_agent::{
    ActionRecord, ReActAgent, ReActConfig, ReActConfigBuilder, ReActError, ReActResponse,
    ReActState, ReActStep, ThoughtAction,
};

// LLM integration exports
pub use rig_integration::{
    ChatMessage, CompletionClient, FinishReason, LlmConfig, LlmError, LlmResponse, MessageRole,
    Provider, RigLlmClient, TokenUsage, ToolCall, ToolCallResponse, ToolCallingClient,
};

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
