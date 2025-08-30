#!/bin/bash
set -e

echo "🔧 Fixing compilation and style issues..."

# Auto-fix clippy warnings
echo "📋 Running clippy auto-fix..."
cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged 2>/dev/null || true

# Format code
echo "🎨 Formatting code..."
cargo fmt --all

# Check compilation
echo "✅ Checking compilation..."
cargo check --all-targets

# Run clippy for remaining warnings
echo "📊 Checking remaining warnings..."
cargo clippy --all-targets --all-features -- -W clippy::all

# Run tests
echo "🧪 Running tests..."
cargo test --all

echo "✨ Done! Check any remaining warnings above."