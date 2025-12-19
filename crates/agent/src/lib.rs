//! LLM Agent implementation using the rig framework.
//!
//! This crate provides:
//! - Agent builder pattern
//! - RAG-enabled agents
//! - Tool definitions
//! - Sales Agent for customer support
//! - CrewAI-style multi-agent orchestration

pub mod builder;
pub mod crew;
pub mod prompts;
pub mod rag_agent;
pub mod sales_agent;
pub mod tools;

pub use builder::AgentBuilder;
pub use rag_agent::RagAgent;
pub use sales_agent::SalesAgentBuilder;

// Re-export crew module components for convenience
pub use crew::{
    Agent as CrewAgent, Crew, CrewBuilder, CrewResult, Flow, FlowBuilder, FlowState, Memory,
    MemoryConfig, MemoryType, Process, ProcessConfig, StateTransition, Task, TaskBuilder,
    TaskOutput, TaskStatus, TransitionCondition,
};
