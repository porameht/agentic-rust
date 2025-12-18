//! Embedding generation for documents.

use async_trait::async_trait;
use common::models::{DocumentChunk, EmbeddedChunk};
use common::{Error, Result};

/// Trait for embedding models
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generate embedding for a single text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Get the embedding dimension
    fn dimension(&self) -> usize;
}

/// Embed document chunks using an embedding model
pub async fn embed_chunks<E: EmbeddingModel>(
    model: &E,
    chunks: Vec<DocumentChunk>,
) -> Result<Vec<EmbeddedChunk>> {
    if chunks.is_empty() {
        return Ok(Vec::new());
    }

    let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
    let embeddings = model.embed_batch(&texts).await?;

    Ok(chunks
        .into_iter()
        .zip(embeddings)
        .map(|(chunk, embedding)| EmbeddedChunk { chunk, embedding })
        .collect())
}

/// Mock embedding model for testing
#[cfg(test)]
pub struct MockEmbeddingModel {
    dimension: usize,
}

#[cfg(test)]
impl MockEmbeddingModel {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[cfg(test)]
#[async_trait]
impl EmbeddingModel for MockEmbeddingModel {
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.0; self.dimension])
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|_| vec![0.0; self.dimension]).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}
