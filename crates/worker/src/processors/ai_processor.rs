//! AI processing utilities.

use common::Result;
use rag_core::chunker::TextChunker;
use rag_core::embeddings::EmbeddingModel;
use rag_core::vector_store::VectorStore;

/// AI processor that combines chunking, embedding, and storage
pub struct AiProcessor<E: EmbeddingModel, V: VectorStore> {
    chunker: TextChunker,
    embedding_model: E,
    vector_store: V,
}

impl<E: EmbeddingModel, V: VectorStore> AiProcessor<E, V> {
    pub fn new(chunker: TextChunker, embedding_model: E, vector_store: V) -> Self {
        Self {
            chunker,
            embedding_model,
            vector_store,
        }
    }

    /// Process and index a document
    pub async fn index_document(
        &self,
        _document_id: uuid::Uuid,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<usize> {
        use common::models::{Document, EmbeddedChunk};

        // Create document for chunking
        let document = Document::new("", content).with_metadata(metadata);

        // Chunk the document
        let chunks = self.chunker.chunk_document(&document);
        let chunk_count = chunks.len();

        if chunks.is_empty() {
            return Ok(0);
        }

        // Generate embeddings
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedding_model.embed_batch(&texts).await?;

        // Create embedded chunks
        let embedded_chunks: Vec<EmbeddedChunk> = chunks
            .into_iter()
            .zip(embeddings)
            .map(|(chunk, embedding)| EmbeddedChunk { chunk, embedding })
            .collect();

        // Store in vector database
        self.vector_store.add_chunks(embedded_chunks).await?;

        Ok(chunk_count)
    }
}
