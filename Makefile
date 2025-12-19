# Agentic Rust Makefile
# =====================

.PHONY: help test test-agent test-crew test-count fmt lint check build clean

# Default target
help:
	@echo "Agentic Rust - Available Commands"
	@echo "=================================="
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run all tests"
	@echo "  make test-agent    - Run Single Agent tests (builder, prompts, rag_agent, sales_agent)"
	@echo "  make test-crew     - Run CrewAI tests (crew module)"
	@echo "  make test-count    - Show test count by module"
	@echo ""
	@echo "Development:"
	@echo "  make fmt           - Format code"
	@echo "  make lint          - Run clippy lints"
	@echo "  make check         - Run cargo check"
	@echo "  make build         - Build the project"
	@echo "  make clean         - Clean build artifacts"
	@echo ""

# ============================================================================
# TESTING
# ============================================================================

# Run all tests
test:
	cargo test --workspace

# Single Agent tests (builder, prompts, rag_agent, sales_agent)
test-agent:
	@echo "=== Single Agent Tests ==="
	cargo test -p agent --lib -- --skip crew::

# CrewAI Multi-Agent tests
test-crew:
	@echo "=== CrewAI Tests ==="
	cargo test -p agent --lib crew

# Show test count by module
test-count:
	@echo "=== Test Count by Module ==="
	@echo ""
	@echo "Single Agent:"
	@echo -n "  builder:      " && cargo test -p agent --lib 2>&1 | grep -c "^test builder::" || echo "0"
	@echo -n "  prompts:      " && cargo test -p agent --lib 2>&1 | grep -c "^test prompts::" || echo "0"
	@echo -n "  rag_agent:    " && cargo test -p agent --lib 2>&1 | grep -c "^test rag_agent::" || echo "0"
	@echo -n "  sales_agent:  " && cargo test -p agent --lib 2>&1 | grep -c "^test sales_agent::" || echo "0"
	@echo ""
	@echo "CrewAI:"
	@echo -n "  crew::agent:       " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::agent::" || echo "0"
	@echo -n "  crew::task:        " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::task::" || echo "0"
	@echo -n "  crew::crew:        " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::crew::" || echo "0"
	@echo -n "  crew::flow:        " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::flow::" || echo "0"
	@echo -n "  crew::memory:      " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::memory::" || echo "0"
	@echo -n "  crew::process:     " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::process::" || echo "0"
	@echo -n "  crew::config:      " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::config::" || echo "0"
	@echo -n "  crew::prompts:     " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::prompts::" || echo "0"
	@echo -n "  crew::tools:       " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::tools::" || echo "0"
	@echo -n "  crew::examples:    " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::examples::" || echo "0"
	@echo -n "  crew::integration: " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::integration::" || echo "0"
	@echo -n "  crew::tests:       " && cargo test -p agent --lib 2>&1 | grep -c "^test crew::tests::" || echo "0"
	@echo ""
	@echo "Total:"
	@cargo test -p agent --lib 2>&1 | grep "test result" | tail -1

# ============================================================================
# DEVELOPMENT
# ============================================================================

# Format code
fmt:
	cargo fmt --all

# Run clippy
lint:
	cargo clippy --workspace -- -D warnings

# Cargo check
check:
	cargo check --workspace

# Build
build:
	cargo build --workspace

# Build release
build-release:
	cargo build --workspace --release

# Clean
clean:
	cargo clean

# ============================================================================
# CI/CD
# ============================================================================

# Run all CI checks
ci: fmt-check lint test
	@echo "All CI checks passed!"

# Check formatting (for CI)
fmt-check:
	cargo fmt --all -- --check

# Security audit
audit:
	cargo audit
