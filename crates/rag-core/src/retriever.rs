//! Document retrieval for RAG.

use crate::embeddings::EmbeddingModel;
use crate::vector_store::VectorStore;
use common::models::SearchResult;
use common::Result;

/// Document retriever that combines embedding and vector search
pub struct Retriever<E: EmbeddingModel, V: VectorStore> {
    embedding_model: E,
    vector_store: V,
    top_k: usize,
    similarity_threshold: f32,
}

impl<E: EmbeddingModel, V: VectorStore> Retriever<E, V> {
    /// Create a new retriever
    pub fn new(embedding_model: E, vector_store: V) -> Self {
        Self {
            embedding_model,
            vector_store,
            top_k: 5,
            similarity_threshold: 0.0,
        }
    }

    /// Set the number of results to retrieve
    pub fn with_top_k(mut self, top_k: usize) -> Self {
        self.top_k = top_k;
        self
    }

    /// Set the minimum similarity threshold
    pub fn with_threshold(mut self, threshold: f32) -> Self {
        self.similarity_threshold = threshold;
        self
    }

    /// Retrieve relevant documents for a query
    pub async fn retrieve(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Generate embedding for query
        let query_embedding = self.embedding_model.embed(query).await?;

        // Search vector store
        let results = self
            .vector_store
            .search(&query_embedding, self.top_k)
            .await?;

        // Filter by similarity threshold
        Ok(results
            .into_iter()
            .filter(|r| r.score >= self.similarity_threshold)
            .collect())
    }

    /// Get reference to the embedding model
    pub fn embedding_model(&self) -> &E {
        &self.embedding_model
    }

    /// Get reference to the vector store
    pub fn vector_store(&self) -> &V {
        &self.vector_store
    }
}
