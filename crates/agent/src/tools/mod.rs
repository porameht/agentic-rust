//! Agent tools for extending agent capabilities.

pub mod search;

// Tool trait for defining custom tools
// This integrates with rig's tool system

use async_trait::async_trait;
use common::Result;
use serde::{Deserialize, Serialize};

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Result of a tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_name: String,
    pub output: String,
    pub success: bool,
}

/// Trait for implementing custom tools
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition
    fn definition(&self) -> ToolDefinition;

    /// Execute the tool with given arguments
    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult>;
}
