//! Prompt configuration management.
//!
//! Load prompts from TOML configuration files for flexible customization.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Prompt template configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub prompt: String,
}

/// Agent configuration from prompts.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPromptConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_k")]
    pub top_k_documents: usize,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub prompts: HashMap<String, PromptTemplate>,
}

/// Root prompt configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptConfig {
    #[serde(default)]
    pub templates: HashMap<String, PromptTemplate>,
    #[serde(default)]
    pub agents: HashMap<String, AgentPromptConfig>,
}

fn default_model() -> String {
    "gpt-4".to_string()
}

fn default_temperature() -> f32 {
    0.7
}

fn default_top_k() -> usize {
    5
}

impl Default for PromptConfig {
    fn default() -> Self {
        Self {
            templates: default_templates(),
            agents: default_agents(),
        }
    }
}

impl PromptConfig {
    /// Load prompt configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::Error::Config(format!("Failed to read prompt config: {}", e)))?;
        Self::from_str(&content)
    }

    /// Load prompt configuration from a TOML string
    pub fn from_str(content: &str) -> crate::Result<Self> {
        toml::from_str(content)
            .map_err(|e| crate::Error::Config(format!("Failed to parse prompt config: {}", e)))
    }

    /// Load from default path (config/prompts.toml) or use defaults
    pub fn load() -> Self {
        let config_paths = [
            "config/prompts.toml",
            "./prompts.toml",
            "/etc/agentic-rust/prompts.toml",
        ];

        for path in config_paths {
            if Path::new(path).exists() {
                match Self::from_file(path) {
                    Ok(config) => {
                        tracing::info!("Loaded prompt config from: {}", path);
                        return config;
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load prompt config from {}: {}", path, e);
                    }
                }
            }
        }

        tracing::info!("Using default prompt configuration");
        Self::default()
    }

    /// Get a template by name
    pub fn get_template(&self, name: &str) -> Option<&PromptTemplate> {
        self.templates.get(name)
    }

    /// Get an agent config by name
    pub fn get_agent(&self, name: &str) -> Option<&AgentPromptConfig> {
        self.agents.get(name)
    }

    /// Get agent prompt for a specific language
    pub fn get_agent_prompt(&self, agent: &str, language: &str) -> Option<&str> {
        self.agents
            .get(agent)
            .and_then(|a| a.prompts.get(language).or_else(|| a.prompts.get("default")))
            .map(|p| p.prompt.as_str())
    }
}

/// Default templates when no config file is found
fn default_templates() -> HashMap<String, PromptTemplate> {
    let mut templates = HashMap::new();

    templates.insert(
        "general_assistant".to_string(),
        PromptTemplate {
            name: "General Assistant".to_string(),
            description: Some("General-purpose AI assistant".to_string()),
            prompt: "You are a helpful AI assistant. You provide accurate, helpful, and concise responses to user questions. When you don't know something, you say so honestly.".to_string(),
        },
    );

    templates.insert(
        "rag_assistant".to_string(),
        PromptTemplate {
            name: "RAG Assistant".to_string(),
            description: Some("RAG-enabled assistant with context awareness".to_string()),
            prompt: r#"You are a helpful AI assistant with access to a knowledge base. When answering questions:
1. Use the provided context to inform your answers
2. If the context doesn't contain relevant information, say so
3. Cite your sources when possible
4. Be accurate and concise"#.to_string(),
        },
    );

    templates.insert(
        "code_assistant".to_string(),
        PromptTemplate {
            name: "Code Assistant".to_string(),
            description: Some("Expert software engineer and coding assistant".to_string()),
            prompt: r#"You are an expert software engineer and coding assistant. You help users with:
- Writing clean, efficient code
- Debugging issues
- Explaining complex concepts
- Suggesting best practices

Always provide working code examples when appropriate."#.to_string(),
        },
    );

    templates.insert(
        "document_qa".to_string(),
        PromptTemplate {
            name: "Document Q&A".to_string(),
            description: Some("Document analysis assistant for Q&A".to_string()),
            prompt: r#"You are a document analysis assistant. Your job is to answer questions based on the provided documents. Guidelines:
1. Only use information from the provided context
2. Quote relevant passages when appropriate
3. If the answer isn't in the documents, clearly state that
4. Summarize complex information clearly"#.to_string(),
        },
    );

    templates
}

/// Default agent configurations
fn default_agents() -> HashMap<String, AgentPromptConfig> {
    let mut agents = HashMap::new();

    // Sales agent
    let mut sales_prompts = HashMap::new();
    sales_prompts.insert(
        "th".to_string(),
        PromptTemplate {
            name: "Thai Sales Agent".to_string(),
            description: None,
            prompt: r#"คุณเป็นผู้ช่วยฝ่ายขายและบริการลูกค้าของบริษัท คุณมีหน้าที่:

1. **ตอบคำถามเกี่ยวกับบริษัท**: ให้ข้อมูลเกี่ยวกับบริษัท บริการ นโยบาย และ FAQ
2. **แนะนำสินค้า**: ช่วยลูกค้าเลือกสินค้าที่เหมาะสมตามความต้องการ งบประมาณ และการใช้งาน
3. **ให้เอกสาร**: จัดหาโบรชัวร์ แคตตาล็อก และเอกสารต่างๆ ให้ลูกค้าดาวน์โหลด
4. **บริการลูกค้า**: ตอบคำถามทั่วไป ช่วยแก้ปัญหา และนำทางลูกค้าไปยังข้อมูลที่ต้องการ

## แนวทางการตอบ:
- พูดคุยเป็นกันเอง สุภาพ เหมือนคุยกับพนักงานขายจริงๆ
- ถามความต้องการของลูกค้าให้ชัดเจนก่อนแนะนำสินค้า
- ให้ข้อมูลที่ถูกต้องและเป็นประโยชน์
- ถ้าไม่แน่ใจ ให้บอกลูกค้าว่าจะตรวจสอบให้
- เมื่อแนะนำสินค้า ให้อธิบายเหตุผลว่าทำไมถึงเหมาะกับลูกค้า
- เสนอเอกสารหรือโบรชัวร์เพิ่มเติมเมื่อเหมาะสม

## เครื่องมือที่มี:
- `product_search`: ค้นหาและแนะนำสินค้า
- `get_brochure`: หาเอกสาร/โบรชัวร์ให้ดาวน์โหลด
- `company_info`: ค้นหาข้อมูลบริษัท FAQ นโยบาย

ตอบเป็นภาษาไทย เว้นแต่ลูกค้าจะใช้ภาษาอื่น"#.to_string(),
        },
    );
    sales_prompts.insert(
        "en".to_string(),
        PromptTemplate {
            name: "English Sales Agent".to_string(),
            description: None,
            prompt: r#"You are a sales and customer service assistant for the company. Your responsibilities are:

1. **Answer company questions**: Provide information about the company, services, policies, and FAQs
2. **Recommend products**: Help customers choose suitable products based on their needs, budget, and use case
3. **Provide documents**: Supply brochures, catalogs, and other documents for customers to download
4. **Customer service**: Answer general questions, help solve problems, and guide customers to the information they need

## Response Guidelines:
- Be friendly and professional, like talking to a real salesperson
- Ask clarifying questions about customer needs before recommending products
- Provide accurate and helpful information
- If unsure, tell the customer you'll check and get back to them
- When recommending products, explain why they're suitable for the customer
- Offer additional documents or brochures when appropriate

## Available Tools:
- `product_search`: Search and recommend products
- `get_brochure`: Find documents/brochures for download
- `company_info`: Search company information, FAQs, policies

Respond in the same language the customer uses."#.to_string(),
        },
    );

    agents.insert(
        "sales".to_string(),
        AgentPromptConfig {
            id: "sales-agent".to_string(),
            name: "Sales Agent".to_string(),
            description: Some("AI assistant for sales support and product recommendations".to_string()),
            default_model: "gpt-4".to_string(),
            temperature: 0.7,
            top_k_documents: 5,
            tools: vec![
                "product_search".to_string(),
                "get_brochure".to_string(),
                "company_info".to_string(),
            ],
            prompts: sales_prompts,
        },
    );

    agents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PromptConfig::default();
        assert!(config.templates.contains_key("general_assistant"));
        assert!(config.agents.contains_key("sales"));
    }

    #[test]
    fn test_get_agent_prompt() {
        let config = PromptConfig::default();
        let prompt = config.get_agent_prompt("sales", "th");
        assert!(prompt.is_some());
        assert!(prompt.unwrap().contains("ผู้ช่วยฝ่ายขาย"));
    }

    #[test]
    fn test_parse_toml() {
        let toml_str = r#"
[templates.test]
name = "Test Template"
prompt = "Test prompt content"

[agents.test]
id = "test-agent"
name = "Test Agent"
default_model = "gpt-3.5-turbo"
temperature = 0.5
tools = ["tool1", "tool2"]

[agents.test.prompts.default]
name = "Default"
prompt = "Default prompt"
"#;

        let config = PromptConfig::from_str(toml_str).unwrap();
        assert!(config.templates.contains_key("test"));
        assert!(config.agents.contains_key("test"));
        assert_eq!(config.agents["test"].temperature, 0.5);
    }
}
