//! Shared constants used across the application.
//!
//! Single source of truth for magic strings and default values.

/// Default LLM model name
pub const DEFAULT_MODEL: &str = "gpt-4";

/// Default temperature for LLM
pub const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Default number of documents for RAG
pub const DEFAULT_TOP_K: usize = 5;

/// Sales agent tools
pub const SALES_AGENT_TOOLS: &[&str] = &["product_search", "get_brochure", "company_info"];

/// Supported languages
pub mod languages {
    pub const THAI: &str = "th";
    pub const ENGLISH: &str = "en";
    pub const DEFAULT: &str = "default";
}

/// Agent IDs
pub mod agents {
    pub const SALES: &str = "sales-agent";
    pub const RAG: &str = "rag-agent";
    pub const CUSTOM: &str = "custom-agent";
}
