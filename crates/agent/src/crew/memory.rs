//! Memory System for Agents and Crews
//!
//! Provides short-term and long-term memory capabilities for agents,
//! allowing them to remember context across tasks and conversations.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Errors that can occur with memory operations
#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Memory storage failed: {0}")]
    StorageFailed(String),

    #[error("Memory retrieval failed: {0}")]
    RetrievalFailed(String),

    #[error("Memory not found: {0}")]
    NotFound(String),

    #[error("Memory capacity exceeded")]
    CapacityExceeded,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Type of memory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryType {
    /// Short-term memory - cleared after task/session
    ShortTerm,

    /// Long-term memory - persisted across sessions
    LongTerm,

    /// Entity memory - stores information about entities (people, places, etc.)
    Entity,

    /// Episodic memory - stores sequences of events
    Episodic,
}

impl Default for MemoryType {
    fn default() -> Self {
        Self::ShortTerm
    }
}

/// Configuration for memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Type of memory
    pub memory_type: MemoryType,

    /// Maximum number of items to store
    pub max_items: usize,

    /// Whether to use embedding-based retrieval
    pub use_embeddings: bool,

    /// TTL for memory items in seconds (None for no expiry)
    pub ttl_seconds: Option<u64>,

    /// Whether to persist memory across sessions
    pub persist: bool,

    /// Path for persistence (if enabled)
    pub storage_path: Option<String>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            memory_type: MemoryType::ShortTerm,
            max_items: 1000,
            use_embeddings: false,
            ttl_seconds: None,
            persist: false,
            storage_path: None,
        }
    }
}

/// A memory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique key for the memory
    pub key: String,

    /// The stored value
    pub value: serde_json::Value,

    /// When this memory was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When this memory was last accessed
    pub last_accessed: chrono::DateTime<chrono::Utc>,

    /// Access count
    pub access_count: usize,

    /// Optional embedding for semantic search
    pub embedding: Option<Vec<f32>>,

    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl MemoryItem {
    /// Create a new memory item
    pub fn new(key: impl Into<String>, value: serde_json::Value) -> Self {
        let now = chrono::Utc::now();
        Self {
            key: key.into(),
            value,
            created_at: now,
            last_accessed: now,
            access_count: 0,
            embedding: None,
            metadata: HashMap::new(),
        }
    }

    /// Mark as accessed
    pub fn touch(&mut self) {
        self.last_accessed = chrono::Utc::now();
        self.access_count += 1;
    }

    /// Check if expired based on TTL
    pub fn is_expired(&self, ttl_seconds: Option<u64>) -> bool {
        if let Some(ttl) = ttl_seconds {
            let age = chrono::Utc::now() - self.created_at;
            age.num_seconds() > ttl as i64
        } else {
            false
        }
    }
}

/// Trait for memory storage backends
#[async_trait]
pub trait MemoryStorage: Send + Sync {
    /// Store a memory item
    async fn store(&self, item: MemoryItem) -> Result<(), MemoryError>;

    /// Retrieve a memory item by key
    async fn retrieve(&self, key: &str) -> Result<Option<MemoryItem>, MemoryError>;

    /// Delete a memory item
    async fn delete(&self, key: &str) -> Result<(), MemoryError>;

    /// Search memories by query
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>, MemoryError>;

    /// List all keys
    async fn keys(&self) -> Result<Vec<String>, MemoryError>;

    /// Clear all memories
    async fn clear(&self) -> Result<(), MemoryError>;

    /// Get the number of stored items
    async fn len(&self) -> Result<usize, MemoryError>;
}

/// In-memory storage implementation
pub struct InMemoryStorage {
    items: RwLock<HashMap<String, MemoryItem>>,
    config: MemoryConfig,
}

impl InMemoryStorage {
    /// Create new in-memory storage
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            items: RwLock::new(HashMap::new()),
            config,
        }
    }
}

#[async_trait]
impl MemoryStorage for InMemoryStorage {
    async fn store(&self, item: MemoryItem) -> Result<(), MemoryError> {
        let mut items = self.items.write().await;

        // Check capacity
        if items.len() >= self.config.max_items && !items.contains_key(&item.key) {
            // Remove oldest item
            if let Some(oldest_key) = items
                .iter()
                .min_by_key(|(_, v)| v.last_accessed)
                .map(|(k, _)| k.clone())
            {
                items.remove(&oldest_key);
            }
        }

        items.insert(item.key.clone(), item);
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> Result<Option<MemoryItem>, MemoryError> {
        let mut items = self.items.write().await;

        if let Some(item) = items.get_mut(key) {
            // Check expiry
            if item.is_expired(self.config.ttl_seconds) {
                items.remove(key);
                return Ok(None);
            }

            item.touch();
            Ok(Some(item.clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete(&self, key: &str) -> Result<(), MemoryError> {
        let mut items = self.items.write().await;
        items.remove(key);
        Ok(())
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>, MemoryError> {
        let items = self.items.read().await;
        let query_lower = query.to_lowercase();

        // Simple text search (in production, use embeddings)
        let mut results: Vec<_> = items
            .values()
            .filter(|item| {
                !item.is_expired(self.config.ttl_seconds)
                    && (item.key.to_lowercase().contains(&query_lower)
                        || item.value.to_string().to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();

        // Sort by relevance (access count as proxy)
        results.sort_by(|a, b| b.access_count.cmp(&a.access_count));
        results.truncate(limit);

        Ok(results)
    }

    async fn keys(&self) -> Result<Vec<String>, MemoryError> {
        let items = self.items.read().await;
        Ok(items.keys().cloned().collect())
    }

    async fn clear(&self) -> Result<(), MemoryError> {
        let mut items = self.items.write().await;
        items.clear();
        Ok(())
    }

    async fn len(&self) -> Result<usize, MemoryError> {
        let items = self.items.read().await;
        Ok(items.len())
    }
}

/// Memory interface for agents
pub struct Memory {
    storage: Arc<dyn MemoryStorage>,
    config: MemoryConfig,
}

impl Memory {
    /// Create a new memory with configuration
    pub fn new(config: MemoryConfig) -> Self {
        let storage = Arc::new(InMemoryStorage::new(config.clone()));
        Self { storage, config }
    }

    /// Create memory with custom storage backend
    pub fn with_storage(storage: Arc<dyn MemoryStorage>, config: MemoryConfig) -> Self {
        Self { storage, config }
    }

    /// Store a value in memory
    pub async fn store(&self, key: &str, value: serde_json::Value) -> Result<(), MemoryError> {
        let item = MemoryItem::new(key, value);
        self.storage.store(item).await
    }

    /// Retrieve a value from memory
    pub async fn retrieve(&self, key: &str) -> Result<Option<serde_json::Value>, MemoryError> {
        Ok(self.storage.retrieve(key).await?.map(|item| item.value))
    }

    /// Delete a value from memory
    pub async fn delete(&self, key: &str) -> Result<(), MemoryError> {
        self.storage.delete(key).await
    }

    /// Search memories
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryItem>, MemoryError> {
        self.storage.search(query, limit).await
    }

    /// Clear all memories
    pub async fn clear(&self) -> Result<(), MemoryError> {
        self.storage.clear().await
    }

    /// Get the number of stored items
    pub async fn len(&self) -> Result<usize, MemoryError> {
        self.storage.len().await
    }

    /// Check if memory is empty
    pub async fn is_empty(&self) -> Result<bool, MemoryError> {
        Ok(self.len().await? == 0)
    }

    /// Get memory configuration
    pub fn config(&self) -> &MemoryConfig {
        &self.config
    }
}

/// Shared crew memory that can be accessed by all agents
pub struct CrewMemory {
    /// Agent-specific memories
    agent_memories: RwLock<HashMap<String, Memory>>,

    /// Shared crew-wide memory
    shared: Memory,

    /// Configuration
    config: MemoryConfig,
}

impl CrewMemory {
    /// Create new crew memory
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            agent_memories: RwLock::new(HashMap::new()),
            shared: Memory::new(config.clone()),
            config,
        }
    }

    /// Get or create memory for an agent
    pub async fn agent_memory(&self, agent_id: &str) -> Memory {
        let memories = self.agent_memories.read().await;
        if memories.contains_key(agent_id) {
            // Create a new memory instance with same config
            // In production, this would return a shared reference
            Memory::new(self.config.clone())
        } else {
            drop(memories);
            let mut memories = self.agent_memories.write().await;
            memories.insert(agent_id.to_string(), Memory::new(self.config.clone()));
            Memory::new(self.config.clone())
        }
    }

    /// Access shared crew memory
    pub fn shared(&self) -> &Memory {
        &self.shared
    }

    /// Store in shared memory
    pub async fn store_shared(
        &self,
        key: &str,
        value: serde_json::Value,
    ) -> Result<(), MemoryError> {
        self.shared.store(key, value).await
    }

    /// Retrieve from shared memory
    pub async fn retrieve_shared(
        &self,
        key: &str,
    ) -> Result<Option<serde_json::Value>, MemoryError> {
        self.shared.retrieve(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_store_retrieve() {
        let memory = Memory::new(MemoryConfig::default());

        memory
            .store("test_key", serde_json::json!({"data": "value"}))
            .await
            .unwrap();

        let result = memory.retrieve("test_key").await.unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap()["data"], "value");
    }

    #[tokio::test]
    async fn test_memory_search() {
        let memory = Memory::new(MemoryConfig::default());

        memory
            .store("user_1", serde_json::json!({"name": "Alice"}))
            .await
            .unwrap();
        memory
            .store("user_2", serde_json::json!({"name": "Bob"}))
            .await
            .unwrap();

        let results = memory.search("Alice", 10).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].key, "user_1");
    }

    #[tokio::test]
    async fn test_memory_clear() {
        let memory = Memory::new(MemoryConfig::default());

        memory
            .store("key1", serde_json::json!("value1"))
            .await
            .unwrap();
        memory
            .store("key2", serde_json::json!("value2"))
            .await
            .unwrap();

        assert_eq!(memory.len().await.unwrap(), 2);

        memory.clear().await.unwrap();

        assert_eq!(memory.len().await.unwrap(), 0);
    }
}
