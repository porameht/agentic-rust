//! Job definitions for background processing.

pub mod embed_document;
pub mod index_document;
pub mod process_chat;

pub use embed_document::EmbedDocumentJob;
pub use index_document::IndexDocumentJob;
pub use process_chat::ProcessChatJob;
