//! Diesel ORM models for database tables.

use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Documents
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = documents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = documents)]
pub struct NewDocument<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub content: &'a str,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Conversations
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = conversations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Conversation {
    pub id: Uuid,
    pub agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = conversations)]
pub struct NewConversation {
    pub id: Uuid,
    pub agent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Messages (with embedded reactions)
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Message {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub reaction_type: Option<String>,
    pub reaction_user_id: Option<String>,
    pub reaction_feedback: Option<String>,
    pub reacted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = messages)]
pub struct NewMessage<'a> {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: &'a str,
    pub content: &'a str,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = messages)]
pub struct MessageReactionUpdate<'a> {
    pub reaction_type: Option<&'a str>,
    pub reaction_user_id: Option<&'a str>,
    pub reaction_feedback: Option<&'a str>,
    pub reacted_at: Option<DateTime<Utc>>,
}

// ============================================================================
// Jobs
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Job {
    pub id: Uuid,
    pub job_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob<'a> {
    pub id: Uuid,
    pub job_type: &'a str,
    pub payload: serde_json::Value,
    pub status: &'a str,
    pub result: Option<serde_json::Value>,
    pub error: Option<&'a str>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Document Chunks
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = document_chunks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct DocumentChunk {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: String,
    pub chunk_index: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = document_chunks)]
pub struct NewDocumentChunk<'a> {
    pub id: Uuid,
    pub document_id: Uuid,
    pub content: &'a str,
    pub chunk_index: i32,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Agents
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = agents)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model: String,
    pub preamble: String,
    pub temperature: f32,
    pub top_k_documents: i32,
    pub tools: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = agents)]
pub struct NewAgent<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub description: Option<&'a str>,
    pub model: &'a str,
    pub preamble: &'a str,
    pub temperature: f32,
    pub top_k_documents: i32,
    pub tools: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Products
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = products)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Product {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub category: String,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<String>,
    pub features: serde_json::Value,
    pub specifications: serde_json::Value,
    pub image_urls: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct<'a> {
    pub id: Uuid,
    pub name: &'a str,
    pub description: &'a str,
    pub category: &'a str,
    pub price: Option<bigdecimal::BigDecimal>,
    pub currency: Option<&'a str>,
    pub features: serde_json::Value,
    pub specifications: serde_json::Value,
    pub image_urls: serde_json::Value,
    pub is_active: bool,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Brochures
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = brochures)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Brochure {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub file_name: String,
    pub file_url: String,
    pub file_type: String,
    pub file_size_bytes: i64,
    pub product_ids: serde_json::Value,
    pub category: String,
    pub language: String,
    pub is_public: bool,
    pub download_count: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = brochures)]
pub struct NewBrochure<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub description: &'a str,
    pub file_name: &'a str,
    pub file_url: &'a str,
    pub file_type: &'a str,
    pub file_size_bytes: i64,
    pub product_ids: serde_json::Value,
    pub category: &'a str,
    pub language: &'a str,
    pub is_public: bool,
    pub download_count: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// FAQs
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = faqs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Faq {
    pub id: Uuid,
    pub question: String,
    pub answer: String,
    pub category: String,
    pub language: String,
    pub is_active: bool,
    pub view_count: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = faqs)]
pub struct NewFaq<'a> {
    pub id: Uuid,
    pub question: &'a str,
    pub answer: &'a str,
    pub category: &'a str,
    pub language: &'a str,
    pub is_active: bool,
    pub view_count: i64,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Company Info
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = company_info)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct CompanyInfo {
    pub id: Uuid,
    pub key: String,
    pub value: String,
    pub category: String,
    pub language: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = company_info)]
pub struct NewCompanyInfo<'a> {
    pub id: Uuid,
    pub key: &'a str,
    pub value: &'a str,
    pub category: &'a str,
    pub language: &'a str,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Product Brochures (Join Table)
// ============================================================================

#[derive(Debug, Clone, Queryable, Selectable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = product_brochures)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ProductBrochure {
    pub product_id: Uuid,
    pub brochure_id: Uuid,
}
