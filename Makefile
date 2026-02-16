# Makefile for OpenAI Agents Rust SDK

.PHONY: help build test check fmt lint doc clean examples

help: ## Show this help message
	@echo "OpenAI Agents Rust SDK - Available commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

test: ## Run all tests
	cargo test

test-verbose: ## Run tests with verbose output
	cargo test -- --nocapture

check: fmt lint test ## Run all checks (format, lint, test)

fmt: ## Format code
	cargo fmt

fmt-check: ## Check code formatting
	cargo fmt --check

lint: ## Run clippy linter
	cargo clippy -- -D warnings

doc: ## Generate documentation
	cargo doc --no-deps --open

doc-build: ## Build documentation without opening
	cargo doc --no-deps

clean: ## Clean build artifacts
	cargo clean

examples: ## List all examples
	@echo "Available examples:"
	@echo "  - hello_world: Basic agent execution"
	@echo "  - tools: Function tools example"
	@echo "  - handoffs: Multi-agent workflow"
	@echo "  - sessions: Conversation history"
	@echo ""
	@echo "Run with: cargo run --example <name>"

run-hello: ## Run hello_world example
	cargo run --example hello_world

run-handoffs: ## Run handoffs example
	cargo run --example handoffs

run-sessions: ## Run sessions example
	cargo run --example sessions

verify: ## Run full verification suite
	@./verify.sh

install-dev: ## Install development dependencies
	rustup component add rustfmt clippy

all: check doc ## Build, test, lint, and generate docs
