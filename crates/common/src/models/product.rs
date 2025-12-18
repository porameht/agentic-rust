//! Product and brochure models for sales agent.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Product information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub features: Vec<String>,
    pub specifications: serde_json::Value,
    pub image_urls: Vec<String>,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Brochure/Document for download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brochure {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub file_name: String,
    pub file_url: String,
    pub file_type: FileType,
    pub file_size_bytes: i64,
    pub product_ids: Vec<Uuid>, // Related products
    pub category: String,
    pub language: String,
    pub is_public: bool,
    pub download_count: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// File type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Pdf,
    Doc,
    Docx,
    Ppt,
    Pptx,
    Xls,
    Xlsx,
    Image,
    Video,
    Other,
}

/// Product recommendation from agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRecommendation {
    pub product: Product,
    pub relevance_score: f32,
    pub reason: String,
}

/// Company information for RAG
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub industry: String,
    pub website: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub address: Option<String>,
    pub faq: Vec<FaqItem>,
    pub policies: serde_json::Value,
    pub metadata: serde_json::Value,
}

/// FAQ item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqItem {
    pub question: String,
    pub answer: String,
    pub category: String,
}

impl Product {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            description: description.into(),
            category: category.into(),
            price: None,
            currency: None,
            features: Vec::new(),
            specifications: serde_json::json!({}),
            image_urls: Vec::new(),
            is_active: true,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_price(mut self, price: f64, currency: impl Into<String>) -> Self {
        self.price = Some(price);
        self.currency = Some(currency.into());
        self
    }

    pub fn with_features(mut self, features: Vec<String>) -> Self {
        self.features = features;
        self
    }
}

impl Brochure {
    pub fn new(
        title: impl Into<String>,
        file_name: impl Into<String>,
        file_url: impl Into<String>,
        file_type: FileType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title: title.into(),
            description: String::new(),
            file_name: file_name.into(),
            file_url: file_url.into(),
            file_type,
            file_size_bytes: 0,
            product_ids: Vec::new(),
            category: String::new(),
            language: "th".to_string(),
            is_public: true,
            download_count: 0,
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}
