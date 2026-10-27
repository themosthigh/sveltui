.PHONY: all build check test run run-hello clean help

# Default target builds all crates and targets
all: build

# Build all workspace crates and examples
build:
	cargo build --workspace --all-targets

# Fast syntax and type checking
check:
	cargo check --workspace --all-targets

# Run tests across all workspace crates
test:
	cargo test --workspace

# Run the centered "Hello, world!" example with reactive signals and repaint on save
run: run-hello

run-hello:
	cargo run --example hello_world

# Clean build artifacts
clean:
	cargo clean

# Display available commands
help:
	@echo "Available make targets:"
	@echo "  make build      - Build all workspace crates and examples"
	@echo "  make check      - Type-check the workspace"
	@echo "  make test       - Run all workspace unit tests"
	@echo "  make run        - Run the hello_world example"
	@echo "  make clean      - Clean build artifacts"
