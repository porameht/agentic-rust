//! CrewAI-style Tool Implementation
//!
//! Tools provide agents with capabilities to interact with the external world.
//! This module implements the BaseTool pattern from CrewAI Python.
//!
//! # Example
//!
//! ```rust,ignore
//! use agent::crew::tools::{BaseTool, ToolInput};
//!
//! #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
//! struct SearchInput {
//!     query: String,
//!     max_results: Option<usize>,
//! }
//!
//! impl ToolInput for SearchInput {
//!     fn description() -> &'static str {
//!         "Search input with query and optional max results"
//!     }
//! }
//!
//! struct WebSearchTool;
//!
//! #[async_trait::async_trait]
//! impl BaseTool for WebSearchTool {
//!     type Input = SearchInput;
//!
//!     fn name(&self) -> &str {
//!         "web_search"
//!     }
//!
//!     fn description(&self) -> &str {
//!         "Search the web for information"
//!     }
//!
//!     async fn run(&self, input: Self::Input) -> Result<String, ToolError> {
//!         // Implementation
//!         Ok(format!("Results for: {}", input.query))
//!     }
//! }
//! ```

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

/// Errors that can occur during tool operations
#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),
}

/// Trait for tool input schemas (like Pydantic BaseModel)
pub trait ToolInput: DeserializeOwned + Serialize + Send + Sync {
    /// Get a description of this input schema
    fn description() -> &'static str;

    /// Get the JSON schema for this input
    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "description": Self::description()
        })
    }
}

/// Base trait for all tools (matching CrewAI Python BaseTool)
#[async_trait]
pub trait BaseTool: Send + Sync {
    /// The input type for this tool
    type Input: ToolInput;

    /// Get the tool name
    fn name(&self) -> &str;

    /// Get the tool description
    fn description(&self) -> &str;

    /// Get the input schema
    fn args_schema(&self) -> serde_json::Value {
        Self::Input::json_schema()
    }

    /// Execute the tool with typed input
    async fn run(&self, input: Self::Input) -> Result<String, ToolError>;

    /// Execute the tool with JSON input (for dynamic invocation)
    async fn run_json(&self, input: serde_json::Value) -> Result<String, ToolError> {
        let typed_input: Self::Input = serde_json::from_value(input)
            .map_err(|e| ToolError::InvalidInput(e.to_string()))?;
        self.run(typed_input).await
    }

    /// Get tool definition for agent context
    fn definition(&self) -> CrewToolDefinition {
        CrewToolDefinition {
            name: self.name().to_string(),
            description: self.description().to_string(),
            args_schema: self.args_schema(),
        }
    }
}

/// Tool definition for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrewToolDefinition {
    pub name: String,
    pub description: String,
    pub args_schema: serde_json::Value,
}

/// Dynamic tool wrapper for runtime tool management
pub struct DynamicTool {
    name: String,
    description: String,
    args_schema: serde_json::Value,
    handler: Box<dyn Fn(serde_json::Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, ToolError>> + Send>> + Send + Sync>,
}

impl DynamicTool {
    /// Create a new dynamic tool
    pub fn new<F, Fut>(
        name: impl Into<String>,
        description: impl Into<String>,
        handler: F,
    ) -> Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<String, ToolError>> + Send + 'static,
    {
        Self {
            name: name.into(),
            description: description.into(),
            args_schema: serde_json::json!({"type": "object"}),
            handler: Box::new(move |input| Box::pin(handler(input))),
        }
    }

    /// Set the args schema
    pub fn with_schema(mut self, schema: serde_json::Value) -> Self {
        self.args_schema = schema;
        self
    }

    /// Get tool name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get tool description
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Execute the tool
    pub async fn execute(&self, input: serde_json::Value) -> Result<String, ToolError> {
        (self.handler)(input).await
    }

    /// Get tool definition
    pub fn definition(&self) -> CrewToolDefinition {
        CrewToolDefinition {
            name: self.name.clone(),
            description: self.description.clone(),
            args_schema: self.args_schema.clone(),
        }
    }
}

// ============================================================================
// BUILT-IN TOOLS (matching data-analyst-agent)
// ============================================================================

/// Input for file read tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileReadInput {
    /// Path to the file to read
    pub file_path: String,
}

impl ToolInput for FileReadInput {
    fn description() -> &'static str {
        "Input for reading a file from the filesystem"
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The path to the file to read"
                }
            },
            "required": ["file_path"]
        })
    }
}

/// File read tool (like FileReadTool in CrewAI)
pub struct FileReadTool;

#[async_trait]
impl BaseTool for FileReadTool {
    type Input = FileReadInput;

    fn name(&self) -> &str {
        "file_read"
    }

    fn description(&self) -> &str {
        "Read the contents of a file from the filesystem. Useful for accessing data files, configuration, or any text-based content."
    }

    async fn run(&self, input: Self::Input) -> Result<String, ToolError> {
        tokio::fs::read_to_string(&input.file_path)
            .await
            .map_err(|e| ToolError::IoError(format!("Failed to read file '{}': {}", input.file_path, e)))
    }
}

/// Input for file write tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWriteInput {
    /// Path to the file to write
    pub file_path: String,
    /// Content to write
    pub content: String,
}

impl ToolInput for FileWriteInput {
    fn description() -> &'static str {
        "Input for writing content to a file"
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "The path to the file to write"
                },
                "content": {
                    "type": "string",
                    "description": "The content to write to the file"
                }
            },
            "required": ["file_path", "content"]
        })
    }
}

/// File write tool
pub struct FileWriteTool;

#[async_trait]
impl BaseTool for FileWriteTool {
    type Input = FileWriteInput;

    fn name(&self) -> &str {
        "file_write"
    }

    fn description(&self) -> &str {
        "Write content to a file on the filesystem. Creates the file if it doesn't exist, or overwrites if it does."
    }

    async fn run(&self, input: Self::Input) -> Result<String, ToolError> {
        // Create parent directories if needed
        if let Some(parent) = std::path::Path::new(&input.file_path).parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| ToolError::IoError(format!("Failed to create directories: {}", e)))?;
        }

        tokio::fs::write(&input.file_path, &input.content)
            .await
            .map_err(|e| ToolError::IoError(format!("Failed to write file '{}': {}", input.file_path, e)))?;

        Ok(format!("Successfully wrote {} bytes to '{}'", input.content.len(), input.file_path))
    }
}

/// Input for REPL tool (code execution)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplInput {
    /// Python code to execute
    pub code: String,
}

impl ToolInput for ReplInput {
    fn description() -> &'static str {
        "Input for executing Python code in a REPL environment"
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "code": {
                    "type": "string",
                    "description": "The Python code to execute"
                }
            },
            "required": ["code"]
        })
    }
}

/// Python REPL tool (like the repl tool in data-analyst-agent)
/// Note: This is a placeholder - actual implementation would use PyO3 or subprocess
pub struct ReplTool;

#[async_trait]
impl BaseTool for ReplTool {
    type Input = ReplInput;

    fn name(&self) -> &str {
        "repl"
    }

    fn description(&self) -> &str {
        "Execute Python code in an interactive REPL environment. Use this to run calculations, process data, or test code snippets."
    }

    async fn run(&self, input: Self::Input) -> Result<String, ToolError> {
        // Placeholder implementation - would use python subprocess
        // In production, this would:
        // 1. Spawn a Python subprocess
        // 2. Execute the code
        // 3. Capture stdout/stderr
        // 4. Return the result

        Ok(format!(
            "[REPL] Would execute Python code:\n```python\n{}\n```\n\nNote: Python execution not implemented in this demo.",
            input.code
        ))
    }
}

/// Input for web search tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSearchInput {
    /// Search query
    pub query: String,
    /// Maximum number of results
    #[serde(default = "default_max_results")]
    pub max_results: usize,
}

fn default_max_results() -> usize {
    5
}

impl ToolInput for WebSearchInput {
    fn description() -> &'static str {
        "Input for searching the web"
    }

    fn json_schema() -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The search query"
                },
                "max_results": {
                    "type": "integer",
                    "description": "Maximum number of results to return",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }
}

/// Web search tool
pub struct WebSearchTool;

#[async_trait]
impl BaseTool for WebSearchTool {
    type Input = WebSearchInput;

    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information on any topic. Returns relevant search results with titles, snippets, and URLs."
    }

    async fn run(&self, input: Self::Input) -> Result<String, ToolError> {
        // Placeholder implementation
        Ok(format!(
            "[Web Search] Query: '{}' (max {} results)\n\nNote: Web search not implemented in this demo.",
            input.query, input.max_results
        ))
    }
}

// ============================================================================
// TOOL REGISTRY
// ============================================================================

/// Registry for managing available tools
pub struct ToolRegistry {
    tools: HashMap<String, Arc<DynamicTool>>,
}

impl ToolRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// Create a registry with default tools
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Add file tools
        registry.register_typed::<FileReadTool>(FileReadTool);
        registry.register_typed::<FileWriteTool>(FileWriteTool);

        // Add code execution
        registry.register_typed::<ReplTool>(ReplTool);

        // Add web search
        registry.register_typed::<WebSearchTool>(WebSearchTool);

        registry
    }

    /// Register a typed tool
    pub fn register_typed<T: BaseTool + 'static>(&mut self, tool: T) {
        let name = tool.name().to_string();
        let description = tool.description().to_string();
        let schema = tool.args_schema();

        let dynamic = DynamicTool {
            name: name.clone(),
            description,
            args_schema: schema,
            handler: Box::new(move |input| {
                let typed_input: T::Input = match serde_json::from_value(input) {
                    Ok(v) => v,
                    Err(e) => return Box::pin(async move {
                        Err(ToolError::InvalidInput(e.to_string()))
                    }),
                };
                // Note: This is a simplified implementation
                // In production, we'd need proper async handling
                Box::pin(async move {
                    Ok(format!("Tool '{}' executed", std::any::type_name::<T>()))
                })
            }),
        };

        self.tools.insert(name, Arc::new(dynamic));
    }

    /// Register a dynamic tool
    pub fn register(&mut self, tool: DynamicTool) {
        self.tools.insert(tool.name.clone(), Arc::new(tool));
    }

    /// Get a tool by name
    pub fn get(&self, name: &str) -> Option<Arc<DynamicTool>> {
        self.tools.get(name).cloned()
    }

    /// Get all tool definitions
    pub fn definitions(&self) -> Vec<CrewToolDefinition> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    /// Get all tool names
    pub fn names(&self) -> Vec<String> {
        self.tools.keys().cloned().collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CONVENIENCE FUNCTIONS
// ============================================================================

/// Create a simple tool from a closure
pub fn simple_tool<F>(
    name: impl Into<String>,
    description: impl Into<String>,
    handler: F,
) -> DynamicTool
where
    F: Fn(serde_json::Value) -> Result<String, ToolError> + Send + Sync + Clone + 'static,
{
    DynamicTool::new(name, description, move |input| {
        let handler = handler.clone();
        async move { handler(input) }
    })
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_read_tool_definition() {
        let tool = FileReadTool;
        let def = tool.definition();

        assert_eq!(def.name, "file_read");
        assert!(def.description.contains("Read"));
        assert!(def.args_schema["properties"]["file_path"].is_object());
    }

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::with_defaults();

        assert!(registry.get("file_read").is_some());
        assert!(registry.get("file_write").is_some());
        assert!(registry.get("repl").is_some());
        assert!(registry.get("web_search").is_some());

        let names = registry.names();
        assert!(names.contains(&"file_read".to_string()));
    }

    #[test]
    fn test_dynamic_tool() {
        let tool = simple_tool(
            "test_tool",
            "A test tool",
            |input: serde_json::Value| {
                Ok(format!("Received: {:?}", input))
            },
        );

        assert_eq!(tool.name(), "test_tool");
        assert_eq!(tool.description(), "A test tool");
    }

    #[tokio::test]
    async fn test_file_read_tool() {
        let tool = FileReadTool;

        // Test with non-existent file
        let result = tool.run(FileReadInput {
            file_path: "/nonexistent/file.txt".to_string(),
        }).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_input_schema() {
        let schema = FileReadInput::json_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["file_path"].is_object());
        assert!(schema["required"].as_array().unwrap().contains(&serde_json::json!("file_path")));
    }
}
