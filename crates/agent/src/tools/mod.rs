//! Agent tools for extending agent capabilities.
//!
//! Available tools for Sales Agent:
//! - `product_search`: ค้นหาและแนะนำสินค้า
//! - `get_brochure`: ค้นหาและให้ลิงก์ดาวน์โหลดเอกสาร
//! - `company_info`: ค้นหาข้อมูลบริษัท, FAQ, นโยบาย
//! - `search`: ค้นหาทั่วไปใน knowledge base

pub mod brochure;
pub mod company_info;
pub mod product_search;
pub mod search;

pub use brochure::BrochureTool;
pub use company_info::CompanyInfoTool;
pub use product_search::ProductSearchTool;
pub use search::SearchTool;

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

/// Create all sales agent tools
pub fn create_sales_agent_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(ProductSearchTool::new()),
        Box::new(BrochureTool::new()),
        Box::new(CompanyInfoTool::new()),
        Box::new(SearchTool::new()),
    ]
}
