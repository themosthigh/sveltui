.PHONY: all build check test lint fmt fmt-check verify clean help

# Default target builds all crates and targets
all: build

# Build all workspace crates and examples
build:
	cargo build --workspace --all-targets

# Fast syntax and type checking
check:
	cargo check --workspace --all-targets

# Run linter across all workspace crates and targets
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Format code
fmt:
	cargo fmt

# Verify code formatting
fmt-check:
	cargo fmt --check

# Run full test suite across all workspace crates
test:
	cargo test --workspace

# Full verification: format check, type check, linting, and tests
verify: fmt-check check lint test

# Clean build artifacts
clean:
	cargo clean

# Display available commands
help:
	@echo "Available make targets:"
	@echo "  make build     - Build all workspace crates and examples"
	@echo "  make check     - Type-check the workspace"
	@echo "  make lint      - Run cargo clippy with -D warnings"
	@echo "  make fmt       - Format all code with cargo fmt"
	@echo "  make fmt-check - Verify code formatting"
	@echo "  make test      - Run all workspace unit tests"
	@echo "  make verify    - Run full verification (fmt-check, check, lint, test)"
	@echo "  make clean     - Clean build artifacts"
