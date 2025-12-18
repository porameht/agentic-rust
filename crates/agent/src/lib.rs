//! LLM Agent implementation using the rig framework.
//!
//! This crate provides:
//! - Agent builder pattern
//! - RAG-enabled agents
//! - Tool definitions
//! - Sales Agent for customer support

pub mod builder;
pub mod prompts;
pub mod rag_agent;
pub mod sales_agent;
pub mod tools;

pub use builder::AgentBuilder;
pub use rag_agent::RagAgent;
pub use sales_agent::SalesAgentBuilder;
