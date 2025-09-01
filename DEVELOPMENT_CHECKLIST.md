# Development Checklist

## After Every Major Change

Run this checklist after making significant code changes to ensure code quality:

### 1. Fix Compilation Errors
```bash
# Check for compilation errors
cargo check --all-targets

# Fix any errors shown
```

### 2. Fix Clippy Warnings
```bash
# Run clippy with all targets and features
cargo clippy --all-targets --all-features -- -W clippy::all

# Auto-fix what's possible
cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged

# Common fixes:
# - Remove unused imports
# - Prefix unused variables with underscore
# - Fix deprecated method calls
```

### 3. Format Code
```bash
# Format all Rust code
cargo fmt --all

# Check formatting without changing files
cargo fmt --all -- --check
```

### 4. Run Tests
```bash
# Run all tests
cargo test --all

# Run tests with output
cargo test --all -- --nocapture

# Run specific test
cargo test test_name
```

### 5. Build Release Version
```bash
# Build optimized version to catch any release-only issues
cargo build --release
```

### 6. Check Documentation
```bash
# Build and check documentation
cargo doc --no-deps --open

# Check for doc warnings
cargo doc --no-deps
```

## Common Fixes for Warnings

### Unused Imports
```rust
// Remove the import or prefix with underscore
use std::collections::HashMap; // Remove if unused
```

### Unused Variables
```rust
// Prefix with underscore
let _unused_var = value;

// Or in function parameters
fn example(_unused_param: Type) {}
```

### Deprecated Methods
```rust
// Replace drag_released with drag_stopped
response.drag_stopped() // Instead of drag_released()
```

### Mutable Variables That Don't Need to Be
```rust
// Remove mut keyword
let value = something; // Instead of let mut value
```

## Automated Fix Script

Create a file `fix_all.sh`:

```bash
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
```

Make it executable:
```bash
chmod +x fix_all.sh
./fix_all.sh
```

## VS Code Settings

Add to `.vscode/settings.json`:

```json
{
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.allTargets": true,
  "rust-analyzer.checkOnSave.extraArgs": [
    "--all-features"
  ],
  "editor.formatOnSave": true,
  "[rust]": {
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

## Pre-commit Hook

Create `.git/hooks/pre-commit`:

```bash
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check

# Clippy check
cargo clippy --all-targets --all-features -- -D warnings

# Test check
cargo test --all

echo "All checks passed!"
```

Make it executable:
```bash
chmod +x .git/hooks/pre-commit
```

## Quick Commands Reference

```bash
# Quick fix most issues
cargo clippy --fix --all-targets --all-features --allow-dirty --allow-staged && cargo fmt --all

# Check everything without fixing
cargo check --all-targets && cargo clippy --all-targets --all-features && cargo fmt --all -- --check

# Run all tests with output
cargo test --all -- --nocapture

# Build and run
cargo build && cargo run

# Clean build
cargo clean && cargo build
```