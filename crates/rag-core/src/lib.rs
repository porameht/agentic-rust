//! RAG (Retrieval-Augmented Generation) core functionality.
//!
//! This crate provides the core components for building RAG systems:
//! - Text chunking strategies
//! - Embedding generation
//! - Vector store abstraction
//! - Document retrieval

pub mod chunker;
pub mod embeddings;
pub mod retriever;
pub mod vector_store;

pub use chunker::TextChunker;
pub use retriever::Retriever;
pub use vector_store::VectorStore;
