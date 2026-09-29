#!/bin/bash

echo "Running screenshot-app tests..."
echo "================================"

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
cargo fmt --check

echo ""
echo "Running clippy lints..."
cargo clippy -- -D warnings

echo ""
echo "Test summary complete!"