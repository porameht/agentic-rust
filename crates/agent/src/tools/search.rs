//! Search tool for agents.

use super::{Tool, ToolDefinition, ToolResult};
use async_trait::async_trait;
use common::Result;
use serde::{Deserialize, Serialize};

/// Search tool parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchParams {
    pub query: String,
    pub limit: Option<usize>,
}

/// Search tool for querying the knowledge base
pub struct SearchTool {
    // In production, this would hold a reference to the retriever
}

impl SearchTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for SearchTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for SearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "search".to_string(),
            description: "Search the knowledge base for relevant documents".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "The search query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of results to return",
                        "default": 5
                    }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let params: SearchParams = serde_json::from_value(args)?;

        // In production, this would query the vector store
        let output = format!(
            "Searched for '{}' with limit {}",
            params.query,
            params.limit.unwrap_or(5)
        );

        Ok(ToolResult {
            tool_name: "search".to_string(),
            output,
            success: true,
        })
    }
}
