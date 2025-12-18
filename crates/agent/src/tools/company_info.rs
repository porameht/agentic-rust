//! Company information tool for sales agent.

use super::{Tool, ToolDefinition, ToolResult};
use async_trait::async_trait;
use common::Result;
use serde::{Deserialize, Serialize};

/// Company info query parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfoParams {
    /// What information to retrieve
    pub info_type: CompanyInfoType,
    /// Specific topic or question
    pub topic: Option<String>,
}

/// Types of company information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompanyInfoType {
    /// General company info (about us)
    About,
    /// Contact information
    Contact,
    /// Frequently asked questions
    Faq,
    /// Company policies (return, warranty, etc.)
    Policy,
    /// Service information
    Service,
    /// Branch/location information
    Location,
    /// Working hours
    Hours,
    /// General search
    General,
}

/// Company information lookup tool
pub struct CompanyInfoTool {
    // In production:
    // - Company info repository
    // - FAQ vector store
}

impl CompanyInfoTool {
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for CompanyInfoTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Tool for CompanyInfoTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "company_info".to_string(),
            description: "ค้นหาข้อมูลเกี่ยวกับบริษัท รวมถึง FAQ, นโยบาย, ข้อมูลติดต่อ, บริการต่างๆ".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "info_type": {
                        "type": "string",
                        "enum": ["about", "contact", "faq", "policy", "service", "location", "hours", "general"],
                        "description": "ประเภทข้อมูลที่ต้องการ"
                    },
                    "topic": {
                        "type": "string",
                        "description": "หัวข้อหรือคำถามเฉพาะ (optional)"
                    }
                },
                "required": ["info_type"]
            }),
        }
    }

    async fn execute(&self, args: serde_json::Value) -> Result<ToolResult> {
        let params: CompanyInfoParams = serde_json::from_value(args)?;

        // TODO: Implement company info lookup
        // 1. Query company info database
        // 2. If FAQ, search vector store
        // 3. Return relevant information

        let output = serde_json::json!({
            "info_type": format!("{:?}", params.info_type),
            "topic": params.topic,
            "data": {},
            "message": "Company info will be retrieved from knowledge base"
        });

        Ok(ToolResult {
            tool_name: "company_info".to_string(),
            output: serde_json::to_string(&output)?,
            success: true,
        })
    }
}
