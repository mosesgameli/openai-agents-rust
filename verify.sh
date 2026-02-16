#!/bin/bash
# Verification script for openai-agents-rust

set -e

echo "🔍 Verifying OpenAI Agents Rust SDK..."
echo ""

# Change to project directory
cd ~/Harnicode/openai-agents-rust

echo "📦 Building project..."
cargo build

echo ""
echo "✅ Build successful!"
echo ""

echo "🧪 Running tests..."
cargo test

echo ""
echo "✅ All tests passed!"
echo ""

echo "📚 Checking documentation..."
cargo doc --no-deps

echo ""
echo "✅ Documentation generated!"
echo ""

echo "🎯 Checking code formatting..."
cargo fmt --check

echo ""
echo "✅ Code is properly formatted!"
echo ""

echo "🔍 Running clippy..."
cargo clippy -- -D warnings

echo ""
echo "✅ No clippy warnings!"
echo ""

echo "🎉 All verifications passed!"
echo ""
echo "To run examples (requires OPENAI_API_KEY):"
echo "  export OPENAI_API_KEY=sk-..."
echo "  cargo run --example hello_world"
echo "  cargo run --example handoffs"
echo "  cargo run --example sessions"
