//! Background worker service for async job processing.
//!
//! This crate provides:
//! - Job definitions for various AI processing tasks
//! - Job processors using apalis
//! - Queue management

pub mod jobs;
pub mod processors;
pub mod queue;

pub use jobs::{EmbedDocumentJob, IndexDocumentJob, ProcessChatJob};
