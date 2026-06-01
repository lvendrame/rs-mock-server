# Makefile for rs-mock-server

.PHONY: help test test-watch build run clean clippy fmt fmt-check coverage setup-hooks install-hooks

# Default target
help:
	@echo "Available commands:"
	@echo "  test         - Run all tests"
	@echo "  test-watch   - Run tests in watch mode"
	@echo "  build        - Build the project"
	@echo "  run          - Run the application"
	@echo "  clean        - Clean build artifacts"
	@echo "  clippy       - Run Clippy linter"
	@echo "  fmt          - Format code"
	@echo "  fmt-check    - Check code formatting"
	@echo "  coverage     - Run tests with function coverage threshold"
	@echo "  setup-hooks  - Install Git pre-commit hooks"
	@echo "  check-all    - Run all checks (tests, clippy, formatting)"

# Test commands
test:
	@echo "🧪 Running tests..."
	cargo test

test-watch:
	@echo "🧪 Running tests in watch mode..."
	cargo watch -x test

# Build commands
build:
	@echo "🔨 Building project..."
	cargo build

build-release:
	@echo "🔨 Building project in release mode..."
	cargo build --release

# Run commands
run:
	@echo "🚀 Running application..."
	cargo run

# Maintenance commands
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean

# Code quality commands
clippy:
	@echo "🔍 Running Clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	@echo "📝 Formatting code..."
	cargo fmt --all

fmt-check:
	@echo "📝 Checking code formatting..."
	cargo fmt --all -- --check

coverage:
	@echo "📊 Running coverage..."
	cargo llvm-cov --workspace --summary-only --ignore-filename-regex 'src/(main|handlers/graphql_handlers)\.rs' --fail-under-functions 95

# Git hooks setup
setup-hooks:
	@echo "🔧 Setting up Git hooks..."
	./scripts/setup-git-hooks.sh

install-hooks: setup-hooks

# Run all checks (like pre-commit)
check-all: test clippy fmt-check
	@echo "✅ All checks passed!"

# Development workflow
dev-setup: setup-hooks
	@echo "🛠️  Development environment setup complete!"
	@echo "💡 Use 'make test-watch' for continuous testing during development"

# CI-like checks
ci: test clippy fmt-check
	@echo "🤖 CI checks completed!"
