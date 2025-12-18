//! Background worker service for async job processing.
//!
//! This crate provides:
//! - Job definitions for various AI processing tasks
//! - Job processors using apalis
//! - Queue management
//! - Job consumer for processing queued jobs

pub mod consumer;
pub mod jobs;
pub mod processors;
pub mod queue;

pub use consumer::{JobConsumer, WorkerState};
pub use jobs::{EmbedDocumentJob, IndexDocumentJob, ProcessChatJob};
