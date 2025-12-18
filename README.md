# Agentic-Rust

A Rust monorepo for building Agent RAG (Retrieval-Augmented Generation) systems with LLM capabilities. This project provides an API service and background worker for AI processing, using the [rig](https://rig.rs/) framework for LLM agent functionality.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         API Service (Axum)                          │
│         /chat  |  /documents  |  /agents  |  /health               │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────────┐
│   PostgreSQL    │    │   Redis Queue   │    │   Qdrant Vector DB  │
└─────────────────┘    └────────┬────────┘    └─────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Worker Service (Apalis)                        │
│   ProcessChatJob  |  EmbedDocumentJob  |  IndexDocumentJob          │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    RAG Agent (rig + Qdrant)                         │
│        Embedding  |  Retrieval  |  LLM Completion                  │
└─────────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
agentic-rust/
├── Cargo.toml              # Workspace root
├── docker-compose.yml      # Local development services
├── .env.example            # Environment variables template
├── migrations/             # Database migrations
└── crates/
    ├── common/             # Shared types, traits, utilities
    ├── rag-core/           # RAG engine (chunking, embeddings, retrieval)
    ├── agent/              # LLM agent implementation with rig
    ├── api/                # REST API service (Axum)
    ├── worker/             # Background job processing (Apalis)
    └── db/                 # Database layer (SQLx + PostgreSQL)
```

## Technology Stack

- **Language**: Rust
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **LLM Framework**: [rig](https://rig.rs/) - Build powerful LLM applications in Rust
- **Vector Database**: [Qdrant](https://qdrant.tech/)
- **Database**: PostgreSQL with SQLx
- **Job Queue**: [Apalis](https://github.com/geofmureithi/apalis) + Redis
- **Async Runtime**: Tokio

## Getting Started

### Prerequisites

- Rust 1.75+ (stable)
- Docker & Docker Compose
- PostgreSQL (or use Docker)
- Redis (or use Docker)
- Qdrant (or use Docker)

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd agentic-rust
   ```

2. **Start development services**
   ```bash
   docker-compose up -d
   ```

3. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env with your API keys
   ```

4. **Run database migrations**
   ```bash
   cargo install sqlx-cli
   sqlx migrate run
   ```

5. **Build the project**
   ```bash
   cargo build
   ```

### Running

**Start the API server:**
```bash
cargo run --bin api-server
```

**Start the worker:**
```bash
cargo run --bin worker
```

## API Endpoints

### Chat
- `POST /api/v1/chat` - Synchronous chat with RAG
- `POST /api/v1/chat/async` - Async chat (returns job_id)
- `GET /api/v1/chat/jobs/:job_id` - Get async job status

### Documents
- `POST /api/v1/documents` - Create document
- `GET /api/v1/documents` - List documents
- `GET /api/v1/documents/:id` - Get document
- `DELETE /api/v1/documents/:id` - Delete document
- `POST /api/v1/documents/:id/index` - Index document for RAG
- `POST /api/v1/documents/search` - Semantic search

### Health
- `GET /health` - Basic health check
- `GET /ready` - Readiness check (verifies dependencies)

## Configuration

Configuration is loaded from environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://agentic:agentic@localhost:5432/agentic` |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `QDRANT_URL` | Qdrant server URL | `http://localhost:6333` |
| `OPENAI_API_KEY` | OpenAI API key | - |
| `SERVER_HOST` | API server host | `0.0.0.0` |
| `SERVER_PORT` | API server port | `8080` |
| `WORKER_CONCURRENCY` | Number of worker threads | `4` |
| `RUST_LOG` | Log level | `info` |

## Crate Overview

### `common`
Shared types, error handling, and configuration management.

### `rag-core`
Core RAG functionality:
- Text chunking with configurable size and overlap
- Embedding model abstraction
- Vector store abstraction (Qdrant, in-memory)
- Document retrieval with similarity search

### `agent`
LLM agent implementation using rig:
- Agent builder pattern
- RAG-enabled agents with dynamic context
- Tool support
- Prompt templates

### `api`
REST API service:
- Axum-based HTTP server
- Chat endpoints (sync/async)
- Document management
- Health checks

### `worker`
Background job processing:
- Chat processing jobs
- Document embedding jobs
- Document indexing pipeline
- Apalis-based job queue

### `db`
Database layer:
- PostgreSQL connection pool
- Repository pattern
- SQLx migrations

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

## License

MIT

## Resources

- [Rig Documentation](https://docs.rig.rs/)
- [Rig GitHub](https://github.com/0xPlaygrounds/rig)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Axum Documentation](https://docs.rs/axum)
- [Apalis Documentation](https://docs.rs/apalis)
