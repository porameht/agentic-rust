# Agentic RAG System - Architecture & Implementation Plan

## Overview

This document outlines the architecture and implementation plan for building an **Agent RAG (Retrieval-Augmented Generation)** system in Rust using a monorepo structure. The system consists of an **API service** and **Worker consumers** for AI processing, leveraging the **rig** library for LLM agent functionality.

## Technology Stack

### Core Technologies
- **Language**: Rust 1.75+ (stable)
- **Async Runtime**: Tokio
- **Web Framework**: Axum 0.8 (ergonomic, modular, built on Tokio/Tower/Hyper)
- **LLM Agent Framework**: [rig-core](https://github.com/0xPlaygrounds/rig) 0.23 - Rust library for LLM applications
- **Vector Database**: Qdrant 1.15 (high-performance, written in Rust)
- **Message Queue**: Redis 0.27 (with `apalis` 0.7 for job processing)
- **Database**: PostgreSQL with Diesel ORM 2.2 (r2d2 connection pooling)
- **Object Storage**: RustFS (S3-compatible)
- **Prompt Management**: Langfuse (optional, for prompt version control)
- **Serialization**: Serde

### Key Dependencies
```toml
# LLM & AI
rig-core = "0.23"             # LLM agent framework
langfuse-ergonomic = "0.6"    # Prompt management

# Web Framework
axum = { version = "0.8", features = ["macros", "multipart"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "compression-gzip"] }

# Async Runtime
tokio = { version = "1", features = ["full"] }

# Database (Diesel ORM)
diesel = { version = "2.2", features = ["postgres", "uuid", "chrono", "serde_json", "r2d2"] }
diesel_migrations = "2.2"

# Redis & Job Queue
redis = { version = "0.27", features = ["tokio-comp", "connection-manager", "aio"] }
deadpool-redis = "0.18"
apalis = "0.7"
apalis-redis = "0.7"

# Vector Database
qdrant-client = "1.15"

# Serialization & Utils
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2"

# Observability
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

---

## Monorepo Structure

```
agentic-rust/
├── Cargo.toml                    # Workspace root
├── .env.example                  # Environment variables template
├── docker-compose.yml            # Local development services
│
├── crates/
│   ├── common/                   # Shared types, traits, utilities
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs         # Configuration management
│   │       ├── error.rs          # Common error types
│   │       ├── models/           # Shared domain models
│   │       │   ├── mod.rs
│   │       │   ├── document.rs   # Document/knowledge models
│   │       │   ├── job.rs        # Job/task models
│   │       │   └── agent.rs      # Agent configuration models
│   │       └── traits.rs         # Shared traits
│   │
│   ├── rag-core/                 # RAG engine core logic
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── embeddings.rs     # Embedding generation
│   │       ├── vector_store.rs   # Vector store abstraction
│   │       ├── retriever.rs      # Document retrieval logic
│   │       ├── chunker.rs        # Text chunking strategies
│   │       └── indexer.rs        # Document indexing
│   │
│   ├── agent/                    # LLM Agent implementation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builder.rs        # Agent builder pattern
│   │       ├── rag_agent.rs      # RAG-enabled agent
│   │       ├── tools/            # Agent tools
│   │       │   ├── mod.rs
│   │       │   ├── search.rs     # Search tool
│   │       │   └── calculator.rs # Example tool
│   │       └── prompts.rs        # Prompt templates
│   │
│   ├── api/                      # REST API service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # API entry point
│   │       ├── lib.rs
│   │       ├── routes/           # API routes
│   │       │   ├── mod.rs
│   │       │   ├── chat.rs       # Chat endpoints
│   │       │   ├── documents.rs  # Document management
│   │       │   ├── agents.rs     # Agent management
│   │       │   └── health.rs     # Health checks
│   │       ├── middleware/       # Axum middleware
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   └── logging.rs
│   │       ├── handlers/         # Request handlers
│   │       │   ├── mod.rs
│   │       │   ├── chat.rs
│   │       │   └── documents.rs
│   │       └── state.rs          # Application state
│   │
│   ├── worker/                   # Background job worker
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs           # Worker entry point
│   │       ├── lib.rs
│   │       ├── jobs/             # Job definitions
│   │       │   ├── mod.rs
│   │       │   ├── embed_document.rs    # Document embedding job
│   │       │   ├── process_chat.rs      # Async chat processing
│   │       │   └── index_document.rs    # Document indexing job
│   │       ├── processors/       # Job processors
│   │       │   ├── mod.rs
│   │       │   └── ai_processor.rs
│   │       └── queue.rs          # Queue management
│   │
│   ├── db/                       # Database layer (Diesel ORM)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── pool.rs           # Connection pool (r2d2)
│   │       ├── models.rs         # Diesel ORM models
│   │       ├── schema.rs         # Auto-generated schema
│   │       ├── repositories/     # Data access layer
│   │       │   ├── mod.rs
│   │       │   ├── document.rs
│   │       │   ├── conversation.rs
│   │       │   ├── message.rs
│   │       │   └── job.rs
│   │       └── migrations/       # Diesel migrations
│   │
│   └── storage/                  # S3-compatible object storage
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── client.rs         # RustFS/S3 client
│           ├── config.rs         # Storage configuration
│           ├── models.rs         # Storage models
│           ├── error.rs          # Storage errors
│           └── presigned.rs      # Presigned URLs
│
├── config/
│   └── prompts.toml              # Prompt configuration (Langfuse)
│
└── tests/                        # Integration tests
    ├── api_tests.rs
    └── worker_tests.rs
```

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              CLIENTS                                      │
│                    (Web App, Mobile, CLI, Slack Bot)                     │
└─────────────────────────────────────────┬───────────────────────────────┘
                                          │
                                          ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           API SERVICE (Axum)                             │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐  │
│  │  /chat   │ │/documents│ │/products │ │  /files  │ │   /health    │  │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────────────┘  │
│       │            │            │            │                          │
│       ▼            ▼            ▼            ▼                          │
│  ┌──────────────────────────────────────────────────────────────────┐  │
│  │                    Application State                              │  │
│  │  • DB Pool  • Redis  • Qdrant  • Storage Client  • Agent Reg.    │  │
│  └──────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │
       ┌──────────────────────────┼──────────────────────────┐
       │                │                │                   │
       ▼                ▼                ▼                   ▼
┌────────────┐ ┌─────────────┐ ┌─────────────────┐ ┌─────────────────────┐
│ PostgreSQL │ │ Redis Queue │ │ Qdrant Vector   │ │ RustFS (S3)         │
│  ┌───────┐ │ │ ┌─────────┐ │ │ ┌─────────────┐ │ │ ┌─────────────────┐ │
│  │tables │ │ │ │job_queue│ │ │ │ embeddings  │ │ │ │ brochures/files │ │
│  │ +11   │ │ │ │results  │ │ │ │ vectors     │ │ │ │ presigned URLs  │ │
│  └───────┘ │ │ └─────────┘ │ │ └─────────────┘ │ │ └─────────────────┘ │
└────────────┘ └──────┬──────┘ └─────────────────┘ └─────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                        WORKER SERVICE (Apalis)                           │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                      Job Processors                                 │ │
│  │  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────────┐  │ │
│  │  │ EmbedDocument   │ │ ProcessChat     │ │ IndexDocument       │  │ │
│  │  │ Job             │ │ Job             │ │ Job                 │  │ │
│  │  └────────┬────────┘ └────────┬────────┘ └────────┬────────────┘  │ │
│  │           │                   │                   │               │ │
│  │           └───────────────────┼───────────────────┘               │ │
│  │                               ▼                                    │ │
│  │  ┌────────────────────────────────────────────────────────────┐  │ │
│  │  │                    RAG Agent (rig)                          │  │ │
│  │  │  • Embedding Model  • LLM Model  • Vector Store Index      │  │ │
│  │  │  • Tools  • Preamble/System Prompt  • Dynamic Context      │  │ │
│  │  └────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         LLM PROVIDERS                                    │
│         OpenAI  │  Anthropic (Claude)  │  Cohere  │  Local Models       │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. RAG Core (`crates/rag-core`)

The RAG core handles document processing, embedding, and retrieval.

```rust
// crates/rag-core/src/lib.rs
use rig::embeddings::EmbeddingModel;
use rig::vector_store::VectorStoreIndex;

pub mod chunker;
pub mod embeddings;
pub mod indexer;
pub mod retriever;
pub mod vector_store;

/// Configuration for RAG operations
pub struct RagConfig {
    pub chunk_size: usize,
    pub chunk_overlap: usize,
    pub top_k: usize,
    pub similarity_threshold: f32,
}

/// Document chunk ready for embedding
pub struct DocumentChunk {
    pub id: String,
    pub document_id: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub chunk_index: usize,
}
```

**Key Features:**
- Text chunking with configurable size and overlap
- Embedding generation using rig's embedding models
- Vector store abstraction (Qdrant primary, in-memory for testing)
- Similarity search with metadata filtering

### 2. Agent Module (`crates/agent`)

Implements LLM agents using the rig framework.

```rust
// crates/agent/src/rag_agent.rs
use rig::agent::Agent;
use rig::completion::Prompt;
use rig::providers::openai;

pub struct RagAgentBuilder {
    model: String,
    preamble: String,
    temperature: f32,
    top_k_documents: usize,
    tools: Vec<Box<dyn rig::tool::Tool>>,
}

impl RagAgentBuilder {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            preamble: String::new(),
            temperature: 0.7,
            top_k_documents: 3,
            tools: Vec::new(),
        }
    }

    pub fn preamble(mut self, preamble: impl Into<String>) -> Self {
        self.preamble = preamble.into();
        self
    }

    pub fn with_rag_context<I: VectorStoreIndex>(
        self,
        index: I,
    ) -> RagAgent<I> {
        // Build agent with dynamic_context
    }
}

pub struct RagAgent<I: VectorStoreIndex> {
    inner: Agent<openai::CompletionModel>,
    index: I,
}

impl<I: VectorStoreIndex> RagAgent<I> {
    pub async fn chat(&self, message: &str) -> Result<String, AgentError> {
        // 1. Retrieve relevant documents from vector store
        // 2. Augment prompt with context
        // 3. Generate response using LLM
    }
}
```

### 3. API Service (`crates/api`)

REST API built with Axum.

```rust
// crates/api/src/routes/chat.rs
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct ChatRequest {
    pub message: String,
    pub conversation_id: Option<String>,
    pub agent_id: Option<String>,
}

#[derive(Serialize)]
pub struct ChatResponse {
    pub response: String,
    pub conversation_id: String,
    pub sources: Vec<DocumentSource>,
}

pub async fn chat_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, ApiError> {
    // For sync responses: use agent directly
    // For async responses: queue job and return job_id
}

pub async fn chat_async_handler(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<JobCreatedResponse>, ApiError> {
    // Queue chat job for worker processing
    let job = ProcessChatJob {
        message: request.message,
        conversation_id: request.conversation_id,
        agent_id: request.agent_id,
    };

    state.job_queue.push(job).await?;

    Ok(Json(JobCreatedResponse { job_id: job.id }))
}
```

**API Endpoints:**
- `POST /api/v1/chat` - Synchronous chat
- `POST /api/v1/chat/async` - Async chat (returns job_id)
- `GET /api/v1/chat/jobs/{job_id}` - Get job status/result
- `POST /api/v1/documents` - Upload document
- `POST /api/v1/documents/{id}/index` - Index document for RAG
- `GET /api/v1/documents/search` - Search documents
- `GET /api/v1/agents` - List available agents
- `POST /api/v1/agents` - Create custom agent

### 4. Worker Service (`crates/worker`)

Background job processing with Apalis.

```rust
// crates/worker/src/jobs/process_chat.rs
use apalis::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessChatJob {
    pub job_id: String,
    pub message: String,
    pub conversation_id: Option<String>,
    pub agent_id: Option<String>,
}

pub async fn process_chat(
    job: ProcessChatJob,
    ctx: JobContext,
) -> Result<(), JobError> {
    let state = ctx.data::<WorkerState>()?;

    // 1. Load or create conversation
    let conversation = state.db
        .get_or_create_conversation(&job.conversation_id)
        .await?;

    // 2. Get agent (default or specified)
    let agent = state.agent_registry
        .get(&job.agent_id.unwrap_or_default())
        .await?;

    // 3. Process with RAG agent
    let response = agent.chat(&job.message).await?;

    // 4. Store result
    state.db.save_message(&conversation.id, &job.message, &response).await?;
    state.redis.set_job_result(&job.job_id, &response).await?;

    Ok(())
}

// crates/worker/src/jobs/embed_document.rs
#[derive(Debug, Serialize, Deserialize)]
pub struct EmbedDocumentJob {
    pub document_id: String,
    pub content: String,
    pub metadata: serde_json::Value,
}

pub async fn embed_document(
    job: EmbedDocumentJob,
    ctx: JobContext,
) -> Result<(), JobError> {
    let state = ctx.data::<WorkerState>()?;

    // 1. Chunk document
    let chunks = state.chunker.chunk(&job.content)?;

    // 2. Generate embeddings
    let embeddings = state.embedding_model
        .embed_documents(chunks.iter().map(|c| c.content.as_str()))
        .await?;

    // 3. Store in vector database
    state.vector_store.add_documents(
        chunks.into_iter().zip(embeddings)
            .map(|(chunk, embedding)| DocumentWithEmbedding {
                id: chunk.id,
                document_id: job.document_id.clone(),
                content: chunk.content,
                metadata: chunk.metadata,
                embedding,
            })
    ).await?;

    Ok(())
}
```

---

## Data Flow

### Chat Flow (Synchronous)
```
1. Client sends POST /api/v1/chat { message: "What is Rust?" }
2. API retrieves relevant documents from Qdrant
3. API augments prompt with context
4. API calls LLM via rig
5. API returns response with sources
```

### Chat Flow (Asynchronous)
```
1. Client sends POST /api/v1/chat/async { message: "Analyze this document..." }
2. API creates ProcessChatJob and pushes to Redis queue
3. API returns { job_id: "xxx" }
4. Worker picks up job from queue
5. Worker processes with RAG agent
6. Worker stores result in Redis
7. Client polls GET /api/v1/chat/jobs/{job_id}
8. Client receives result when ready
```

### Document Indexing Flow
```
1. Client sends POST /api/v1/documents with file
2. API stores document metadata in PostgreSQL
3. API creates EmbedDocumentJob and pushes to queue
4. Worker chunks document
5. Worker generates embeddings
6. Worker stores embeddings in Qdrant
7. Document is now searchable via RAG
```

---

## Implementation Plan

### Phase 1: Foundation (Week 1-2)
- [ ] Set up Cargo workspace structure
- [ ] Implement `common` crate (config, errors, models)
- [ ] Implement `db` crate (PostgreSQL connection, basic repositories)
- [ ] Set up Docker Compose for local development
- [ ] Create database migrations

### Phase 2: RAG Core (Week 2-3)
- [ ] Implement text chunking strategies
- [ ] Set up Qdrant vector store integration
- [ ] Implement embedding generation with rig
- [ ] Create vector store abstraction layer
- [ ] Implement document retrieval/similarity search

### Phase 3: Agent Implementation (Week 3-4)
- [ ] Implement basic agent builder
- [ ] Create RAG-enabled agent
- [ ] Add tool support (search, calculator, etc.)
- [ ] Implement prompt templates
- [ ] Add support for multiple LLM providers

### Phase 4: API Service (Week 4-5)
- [ ] Set up Axum server with middleware
- [ ] Implement chat endpoints (sync/async)
- [ ] Implement document management endpoints
- [ ] Add authentication middleware
- [ ] Implement WebSocket for streaming responses

### Phase 5: Worker Service (Week 5-6)
- [ ] Set up Apalis with Redis backend
- [ ] Implement ProcessChatJob
- [ ] Implement EmbedDocumentJob
- [ ] Implement IndexDocumentJob
- [ ] Add job monitoring and retry logic

### Phase 6: Integration & Testing (Week 6-7)
- [ ] Integration tests for full RAG pipeline
- [ ] Load testing
- [ ] Documentation
- [ ] Deployment configuration (Docker, K8s)

---

## Configuration

```toml
# config/default.toml

[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://user:pass@localhost:5432/agentic"
max_connections = 10

[redis]
url = "redis://localhost:6379"

[qdrant]
url = "http://localhost:6333"
collection = "documents"

[llm]
provider = "openai"  # or "anthropic", "cohere"
model = "gpt-4"
embedding_model = "text-embedding-3-small"
temperature = 0.7

[rag]
chunk_size = 1000
chunk_overlap = 200
top_k = 5
similarity_threshold = 0.7

[worker]
concurrency = 4
```

---

## Environment Variables

```bash
# .env.example

# Database
DATABASE_URL=postgres://agentic:agentic@localhost:5432/agentic

# Redis
REDIS_URL=redis://localhost:6379

# Qdrant
QDRANT_URL=http://localhost:6333

# LLM Providers
OPENAI_API_KEY=sk-your-openai-key
ANTHROPIC_API_KEY=sk-ant-your-anthropic-key
COHERE_API_KEY=your-cohere-key

# Langfuse Prompt Management (optional)
LANGFUSE_PUBLIC_KEY=pk-lf-your-public-key
LANGFUSE_SECRET_KEY=sk-lf-your-secret-key
LANGFUSE_BASE_URL=https://cloud.langfuse.com

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Worker
WORKER_CONCURRENCY=4

# RAG Configuration
RAG_CHUNK_SIZE=1000
RAG_CHUNK_OVERLAP=200
RAG_TOP_K=5

# Object Storage (S3-compatible: RustFS)
STORAGE_ENDPOINT=http://localhost:9000
STORAGE_ACCESS_KEY=admin
STORAGE_SECRET_KEY=adminpassword
STORAGE_REGION=us-east-1
STORAGE_PATH_STYLE=true
STORAGE_DEFAULT_BUCKET=brochures

# Logging
RUST_LOG=info,api=debug,worker=debug
```

---

## Docker Compose (Development)

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:16-alpine
    container_name: agentic-postgres
    environment:
      POSTGRES_USER: agentic
      POSTGRES_PASSWORD: agentic
      POSTGRES_DB: agentic
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U agentic"]
      interval: 5s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    container_name: agentic-redis
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5

  qdrant:
    image: qdrant/qdrant:latest
    container_name: agentic-qdrant
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant_data:/qdrant/storage
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:6333/health"]
      interval: 10s
      timeout: 5s
      retries: 5

  # S3-compatible Object Storage (RustFS)
  rustfs:
    image: ghcr.io/rustfs/rustfs:latest
    container_name: agentic-rustfs
    ports:
      - "9000:9000"
    volumes:
      - rustfs_data:/data
    environment:
      RUSTFS_ROOT_USER: admin
      RUSTFS_ROOT_PASSWORD: adminpassword
      RUSTFS_DATA_DIR: /data
      RUSTFS_PORT: 9000
    healthcheck:
      test: ["CMD", "wget", "-q", "--spider", "http://localhost:9000/health"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
  redis_data:
  qdrant_data:
  rustfs_data:
```

---

## Key Design Decisions

### 1. Why Rig for LLM?
- Native Rust library with zero-cost abstractions
- Built-in support for RAG with `VectorStoreIndex` trait
- Provider-agnostic (OpenAI, Anthropic, Cohere)
- Type-safe tool calling
- Ergonomic agent builder pattern

### 2. Why Qdrant for Vector DB?
- Written in Rust (performance, safety)
- High throughput for similarity search
- Rich filtering capabilities
- Payload storage for metadata
- Native rig integration via `rig-qdrant`

### 3. Why Apalis for Job Queue?
- Rust-native job processing
- Redis backend for durability
- Strongly typed job arguments
- Built-in retry and monitoring
- Clean integration with Tokio

### 4. Why Axum for API?
- Built on Tokio/Tower/Hyper (same async ecosystem)
- Middleware reusable with tonic (gRPC) if needed
- Type-safe extractors
- Excellent performance
- Good community support

### 5. Why RustFS for Object Storage?
- Lightweight S3-compatible file server
- Written in Rust (consistent ecosystem)
- Simple deployment and configuration
- Can be replaced with MinIO or AWS S3 in production
- Supports presigned URLs for secure file access

### 6. Why Diesel ORM?
- Compile-time type safety for SQL queries
- Strong Rust type system integration
- Automatic migrations with version control
- r2d2 connection pooling for performance
- Well-maintained with excellent documentation

---

## Sources & References

- [Rig - Build Powerful LLM Applications in Rust](https://rig.rs/)
- [Rig Documentation](https://docs.rs/rig-core/latest/rig/)
- [Rig GitHub](https://github.com/0xPlaygrounds/rig)
- [Build a RAG System with Rig](https://medium.com/@0thTachi/build-a-rag-system-with-rig-in-under-100-lines-of-code-26fce8e017b4)
- [SurrealDB - RAG can be Rigged](https://surrealdb.com/blog/rag-can-be-rigged)
- [Qdrant Vector Database](https://qdrant.tech/)
- [Apalis - Background Job Processing](https://docs.rs/apalis/latest/apalis/)
- [Axum Web Framework](https://docs.rs/axum/latest/axum/)
- [Diesel ORM](https://diesel.rs/)
- [Diesel Getting Started Guide](https://diesel.rs/guides/getting-started)
- [RustFS - S3-compatible Object Storage](https://github.com/rustfs/rustfs)
- [Langfuse - Prompt Management](https://langfuse.com/docs)
- [Rust Microservices Monorepo](https://github.com/jayden-dang/rust-microservices-monorepo)
