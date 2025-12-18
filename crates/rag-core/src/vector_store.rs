//! Vector store abstraction for storing and querying embeddings.

use async_trait::async_trait;
use common::models::{EmbeddedChunk, SearchResult};
use common::Result;

/// Trait for vector store implementations
#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Add embedded chunks to the vector store
    async fn add_chunks(&self, chunks: Vec<EmbeddedChunk>) -> Result<()>;

    /// Search for similar chunks
    async fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<SearchResult>>;

    /// Delete chunks by document ID
    async fn delete_by_document_id(&self, document_id: &uuid::Uuid) -> Result<()>;
}

/// In-memory vector store for testing and development
pub struct InMemoryVectorStore {
    chunks: std::sync::RwLock<Vec<EmbeddedChunk>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            chunks: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Calculate cosine similarity between two vectors
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot_product / (norm_a * norm_b)
    }
}

impl Default for InMemoryVectorStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_chunks(&self, chunks: Vec<EmbeddedChunk>) -> Result<()> {
        let mut store = self.chunks.write().unwrap();
        store.extend(chunks);
        Ok(())
    }

    async fn search(&self, query_embedding: &[f32], top_k: usize) -> Result<Vec<SearchResult>> {
        let store = self.chunks.read().unwrap();

        let mut results: Vec<(SearchResult, f32)> = store
            .iter()
            .map(|embedded| {
                let score = Self::cosine_similarity(query_embedding, &embedded.embedding);
                (
                    SearchResult {
                        chunk: embedded.chunk.clone(),
                        score,
                    },
                    score,
                )
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        Ok(results.into_iter().take(top_k).map(|(r, _)| r).collect())
    }

    async fn delete_by_document_id(&self, document_id: &uuid::Uuid) -> Result<()> {
        let mut store = self.chunks.write().unwrap();
        store.retain(|chunk| &chunk.chunk.document_id != document_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::models::DocumentChunk;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_in_memory_store() {
        let store = InMemoryVectorStore::new();
        let doc_id = Uuid::new_v4();

        let chunks = vec![EmbeddedChunk {
            chunk: DocumentChunk::new(doc_id, "test content", 0),
            embedding: vec![1.0, 0.0, 0.0],
        }];

        store.add_chunks(chunks).await.unwrap();

        let results = store.search(&[1.0, 0.0, 0.0], 1).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!((results[0].score - 1.0).abs() < 0.001);
    }
}
