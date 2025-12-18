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
├── config/
│   └── prompts.toml        # Prompt configuration (Langfuse)
├── migrations/             # Database migrations
└── crates/
    ├── common/             # Shared types, traits, utilities
    ├── rag-core/           # RAG engine (chunking, embeddings, retrieval)
    ├── agent/              # LLM agent implementation with rig
    ├── api/                # REST API service (Axum)
    ├── worker/             # Background job processing (Apalis)
    ├── db/                 # Database layer (SQLx + PostgreSQL)
    └── storage/            # S3-compatible object storage (RustFS)
```

## Technology Stack

- **Language**: Rust
- **Web Framework**: [Axum](https://github.com/tokio-rs/axum)
- **LLM Framework**: [rig](https://rig.rs/) - Build powerful LLM applications in Rust
- **Vector Database**: [Qdrant](https://qdrant.tech/)
- **Database**: PostgreSQL with SQLx
- **Job Queue**: [Apalis](https://github.com/geofmureithi/apalis) + Redis
- **Object Storage**: RustFS (S3-compatible)
- **Async Runtime**: Tokio

## Getting Started

### Prerequisites

- Rust 1.75+ (stable)
- Docker & Docker Compose
- PostgreSQL (or use Docker)
- Redis (or use Docker)
- Qdrant (or use Docker)
- RustFS (or use Docker)

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

### Files (Storage)
- `POST /api/v1/files/brochures` - Upload brochure file
- `POST /api/v1/files/:bucket/:key/upload-url` - Get presigned upload URL
- `GET /api/v1/files/:bucket` - List files in bucket
- `GET /api/v1/files/:bucket/:key` - Get file info
- `GET /api/v1/files/:bucket/:key/download` - Get presigned download URL
- `DELETE /api/v1/files/:bucket/:key` - Delete file

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
| `STORAGE_ENDPOINT` | RustFS/S3 endpoint URL | `http://localhost:9000` |
| `STORAGE_ACCESS_KEY` | Storage access key | `admin` |
| `STORAGE_SECRET_KEY` | Storage secret key | `adminpassword` |
| `STORAGE_DEFAULT_BUCKET` | Default bucket name | `brochures` |

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

### `storage`
S3-compatible object storage:
- RustFS integration (S3-compatible)
- File upload/download with streaming
- Presigned URL support
- Automatic bucket creation
- SHA256 integrity verification

## Deployment

### Local Development

1. **Start infrastructure services**
   ```bash
   docker-compose up -d
   ```

2. **Verify services are running**
   ```bash
   docker-compose ps
   ```

   Services should show:
   - `agentic-postgres` (port 5432)
   - `agentic-redis` (port 6379)
   - `agentic-qdrant` (ports 6333, 6334)
   - `agentic-rustfs` (port 9000)

3. **Run the API and Worker**
   ```bash
   # Terminal 1: API server
   cargo run --bin api-server

   # Terminal 2: Worker
   cargo run --bin worker
   ```

### Docker Deployment

Build and run the entire stack with Docker:

```bash
# Build the Rust services
docker build -t agentic-api -f Dockerfile.api .
docker build -t agentic-worker -f Dockerfile.worker .

# Run with docker-compose (includes all dependencies)
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d
```

### Production Deployment

For production environments:

1. **Environment Variables**
   ```bash
   # Required
   DATABASE_URL=postgres://user:password@host:5432/agentic
   REDIS_URL=redis://host:6379
   QDRANT_URL=http://host:6333
   OPENAI_API_KEY=sk-your-key

   # Storage (RustFS)
   STORAGE_ENDPOINT=http://rustfs-host:9000
   STORAGE_ACCESS_KEY=your-access-key
   STORAGE_SECRET_KEY=your-secret-key
   STORAGE_DEFAULT_BUCKET=brochures
   ```

2. **Database Migrations**
   ```bash
   cargo install sqlx-cli
   sqlx migrate run
   ```

3. **Run Services**
   ```bash
   # Build release binaries
   cargo build --release

   # Run API (with systemd or process manager)
   ./target/release/api-server

   # Run Worker
   ./target/release/worker
   ```

### Kubernetes Deployment

Example Kubernetes manifests:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: agentic-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: agentic-api
  template:
    metadata:
      labels:
        app: agentic-api
    spec:
      containers:
      - name: api
        image: your-registry/agentic-api:latest
        ports:
        - containerPort: 8080
        envFrom:
        - secretRef:
            name: agentic-secrets
        - configMapRef:
            name: agentic-config
---
apiVersion: v1
kind: Service
metadata:
  name: agentic-api
spec:
  selector:
    app: agentic-api
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

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
