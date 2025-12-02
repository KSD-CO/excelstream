.PHONY: help check fmt fmt-check clippy test build build-release clean all ci

# Default target
help:
	@echo "Available targets:"
	@echo "  make check        - Run all checks (format + clippy + test)"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make fmt-check    - Check code formatting without modifying"
	@echo "  make clippy       - Run clippy linter with strict warnings"
	@echo "  make test         - Run all tests"
	@echo "  make build        - Build in debug mode"
	@echo "  make build-release- Build in release mode"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make ci           - Run all CI checks (same as GitHub Actions)"
	@echo "  make all          - Format, build, and test"

# CI checks (same as GitHub Actions)
ci: fmt-check clippy test
	@echo "✅ All CI checks passed!"

# Check everything
check: fmt clippy test
	@echo "✅ All checks passed!"

# Format code
fmt:
	@echo "🔧 Formatting code..."
	@cargo fmt

# Check formatting
fmt-check:
	@echo "🔍 Checking code formatting..."
	@cargo fmt -- --check

# Run clippy
clippy:
	@echo "🔍 Running clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	@echo "🧪 Running tests..."
	@cargo test --all-features

# Build debug
build:
	@echo "🔨 Building (debug mode)..."
	@cargo build --all-features

# Build release
build-release:
	@echo "🔨 Building (release mode)..."
	@cargo build --release --all-features

# Clean
clean:
	@echo "🧹 Cleaning build artifacts..."
	@cargo clean

# Build and test everything
all: fmt build test
	@echo "✅ Build and test completed!"

# Additional useful targets

# Run benchmarks
bench:
	@echo "📊 Running benchmarks..."
	@cargo bench

# Build all examples
examples:
	@echo "🔨 Building all examples..."
	@cargo build --examples --all-features

# Build specific example
example-%:
	@echo "🔨 Building example: $*..."
	@cargo build --example $*

# Run specific example
run-example-%:
	@echo "🚀 Running example: $*..."
	@cargo run --example $*

# Build and test with verbose output
verbose:
	@echo "🔍 Running with verbose output..."
	@cargo build --verbose --all-features
	@cargo test --verbose --all-features

# Check documentation
doc:
	@echo "📚 Building documentation..."
	@cargo doc --all-features --no-deps

# Open documentation in browser
doc-open:
	@echo "📚 Opening documentation..."
	@cargo doc --all-features --no-deps --open

# Update dependencies
update:
	@echo "📦 Updating dependencies..."
	@cargo update

# Check for outdated dependencies
outdated:
	@echo "📦 Checking for outdated dependencies..."
	@cargo outdated || echo "Install cargo-outdated: cargo install cargo-outdated"

# Security audit
audit:
	@echo "🔒 Running security audit..."
	@cargo audit || echo "Install cargo-audit: cargo install cargo-audit"

# Coverage report
coverage:
	@echo "📊 Generating coverage report..."
	@cargo tarpaulin --all-features || echo "Install cargo-tarpaulin: cargo install cargo-tarpaulin"

# Quick check before commit
pre-commit: fmt clippy test
	@echo "✅ Pre-commit checks passed!"

# Quick check before push (same as CI)
pre-push: ci
	@echo "✅ Ready to push!"
