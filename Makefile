# Agentic Rust Makefile
# =====================

.PHONY: help test test-all test-flow1 test-flow2 test-agent test-crew fmt lint check build clean

# Default target
help:
	@echo "Agentic Rust - Available Commands"
	@echo "=================================="
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run all tests"
	@echo "  make test-flow1    - Run Flow 1 (Single Agent) tests only"
	@echo "  make test-flow2    - Run Flow 2 (CrewAI) tests only"
	@echo "  make test-agent    - Run agent crate tests"
	@echo "  make test-crew     - Run crew module tests"
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

test-all: test

# Flow 1: Single Agent tests (builder, prompts, rag_agent, sales_agent)
test-flow1:
	@echo "=== Flow 1: Single Agent Tests ==="
	@cargo test -p agent --lib -- builder:: prompts:: rag_agent:: sales_agent:: 2>&1 | grep -E "^test (builder|prompts|rag_agent|sales_agent)::" || true
	@echo ""
	@echo "Running Flow 1 tests..."
	@cargo test -p agent --lib builder prompts rag_agent sales_agent 2>&1 | tail -1

# Flow 2: CrewAI Multi-Agent tests
test-flow2:
	@echo "=== Flow 2: CrewAI Tests ==="
	@cargo test -p agent --lib -- crew:: 2>&1 | grep -E "^test crew::" | head -20 || true
	@echo "... (more tests)"
	@echo ""
	@cargo test -p agent --lib crew 2>&1 | tail -1

# Agent crate tests
test-agent:
	cargo test -p agent --lib

# Crew module tests only
test-crew:
	cargo test -p agent --lib crew

# Show test count by module
test-count:
	@echo "=== Test Count by Module ==="
	@echo ""
	@echo "Flow 1 (Single Agent):"
	@echo -n "  builder:      " && cargo test -p agent --lib 2>&1 | grep -c "^test builder::" || echo "0"
	@echo -n "  prompts:      " && cargo test -p agent --lib 2>&1 | grep -c "^test prompts::" || echo "0"
	@echo -n "  rag_agent:    " && cargo test -p agent --lib 2>&1 | grep -c "^test rag_agent::" || echo "0"
	@echo -n "  sales_agent:  " && cargo test -p agent --lib 2>&1 | grep -c "^test sales_agent::" || echo "0"
	@echo ""
	@echo "Flow 2 (CrewAI):"
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
