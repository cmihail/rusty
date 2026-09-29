#!/bin/bash

echo "Running universal-search tests..."
echo "================================="

# Run unit tests
echo "Running unit tests..."
cargo test --lib

echo ""
echo "Running integration tests..."
cargo test --test integration_tests

echo ""
echo "Running all tests with verbose output..."
cargo test -- --nocapture

echo ""
echo "Checking code formatting..."
if command -v rustfmt >/dev/null 2>&1; then
    echo "Running cargo fmt --check..."
    if cargo fmt --check; then
        echo "✓ Code formatting is correct"
    else
        echo "✗ Code formatting issues found"
    fi
else
    echo "rustfmt not found - skipping formatting check"
    echo "Install with: rustup component add rustfmt"
fi

echo ""
echo "Running clippy lints..."
if cargo clippy --version >/dev/null 2>&1; then
    echo "Running cargo clippy..."
    if cargo clippy -- -D warnings; then
        echo "✓ No clippy warnings found"
    else
        echo "✗ Clippy warnings found"
    fi
else
    echo "clippy not found - skipping lint check"
    echo "Install with: rustup component add clippy"
fi

echo ""
echo "Building in release mode to check for release-specific issues..."
if cargo build --release; then
    echo "✓ Release build successful"
else
    echo "✗ Release build failed"
fi

echo ""
echo "========================================="
echo "Test summary complete!"
echo "All checks have been performed."
echo "========================================="