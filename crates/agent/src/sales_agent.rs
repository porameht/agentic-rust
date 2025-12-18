//! Sales Agent implementation for customer support and product recommendations.
//!
//! This agent is designed to:
//! - Answer questions about the company
//! - Recommend products based on customer needs
//! - Provide brochures and documents for download
//! - Handle FAQ and policies

use crate::tools::{self, Tool};
use common::models::AgentConfig;

/// Sales agent preamble (system prompt) in Thai
pub const SALES_AGENT_PREAMBLE_TH: &str = r#"คุณเป็นผู้ช่วยฝ่ายขายและบริการลูกค้าของบริษัท คุณมีหน้าที่:

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

ตอบเป็นภาษาไทย เว้นแต่ลูกค้าจะใช้ภาษาอื่น"#;

/// Sales agent preamble in English
pub const SALES_AGENT_PREAMBLE_EN: &str = r#"You are a sales and customer service assistant for the company. Your responsibilities are:

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

Respond in the same language the customer uses."#;

/// Create a sales agent configuration
pub fn create_sales_agent_config(language: &str) -> AgentConfig {
    let preamble = match language {
        "th" | "thai" => SALES_AGENT_PREAMBLE_TH,
        _ => SALES_AGENT_PREAMBLE_EN,
    };

    AgentConfig {
        id: "sales-agent".to_string(),
        name: "Sales Agent".to_string(),
        description: "AI assistant for sales support and product recommendations".to_string(),
        model: "gpt-4".to_string(),
        preamble: preamble.to_string(),
        temperature: 0.7,
        top_k_documents: 5,
        tools: vec![
            "product_search".to_string(),
            "get_brochure".to_string(),
            "company_info".to_string(),
        ],
    }
}

/// Sales agent builder with all tools pre-configured
pub struct SalesAgentBuilder {
    config: AgentConfig,
    tools: Vec<Box<dyn Tool>>,
}

impl SalesAgentBuilder {
    /// Create a new sales agent builder with Thai language
    pub fn new() -> Self {
        Self {
            config: create_sales_agent_config("th"),
            tools: tools::create_sales_agent_tools(),
        }
    }

    /// Set language for the agent
    pub fn language(mut self, language: &str) -> Self {
        self.config = create_sales_agent_config(language);
        self
    }

    /// Set custom model
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature;
        self
    }

    /// Set number of documents to retrieve for RAG
    pub fn top_k_documents(mut self, top_k: usize) -> Self {
        self.config.top_k_documents = top_k;
        self
    }

    /// Add custom preamble (appends to existing)
    pub fn with_custom_context(mut self, context: &str) -> Self {
        self.config.preamble = format!("{}\n\n## Additional Context:\n{}", self.config.preamble, context);
        self
    }

    /// Build the agent configuration
    pub fn build(self) -> (AgentConfig, Vec<Box<dyn Tool>>) {
        (self.config, self.tools)
    }
}

impl Default for SalesAgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sales_agent_builder() {
        let (config, tools) = SalesAgentBuilder::new()
            .language("th")
            .model("gpt-4-turbo")
            .temperature(0.8)
            .build();

        assert_eq!(config.id, "sales-agent");
        assert_eq!(config.model, "gpt-4-turbo");
        assert_eq!(config.temperature, 0.8);
        assert_eq!(tools.len(), 4);
    }
}
