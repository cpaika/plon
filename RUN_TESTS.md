# Running Claude Code Tests

All Claude Code tests are now integrated into the standard Rust test framework and can be run with `cargo test`.

## Running All Tests

```bash
# Run all tests (unit, integration, UI)
cargo test

# Run only Claude Code related tests
cargo test claude_code

# Run with output for debugging
cargo test claude_code -- --nocapture
```

## Test Categories

### Unit Tests

```bash
# Domain model tests
cargo test --lib claude_code::tests

# Service layer tests  
cargo test --lib claude_code_service_tests

# Command executor tests
cargo test --lib command_executor::tests
```

### Integration Tests

```bash
# Full integration tests (requires database)
cargo test --test claude_code_integration_tests

# UI tests
cargo test --test claude_code_ui_tests
```

### Performance Tests

```bash
# Run performance benchmarks
cargo bench claude_code_performance_tests
```

## Test Organization

- **Unit Tests**: Located in the same files as the code they test (`#[cfg(test)]` modules)
- **Integration Tests**: Located in `tests/` directory
- **Mocks**: Implemented in Rust in `src/services/command_executor.rs`
- **Test Utilities**: Located in `tests/claude_code_test_utils.rs`

## Mock System

The tests use a Rust-based mock system instead of external scripts:

- `MockCommandExecutor`: Simulates CLI commands (git, claude, gh)
- Configurable responses and delays
- Call history tracking for assertions
- Thread-safe and async-compatible

## Quick Test Commands

```bash
# Quick smoke test
cargo test --lib claude_code::tests

# Full test suite
cargo test claude_code

# Test with specific mock scenario
cargo test test_launch_claude_code_success

# Test with verbose output
RUST_LOG=debug cargo test claude_code -- --nocapture
```

## Continuous Integration

Add to your CI pipeline:

```yaml
- name: Run Tests
  run: cargo test claude_code
  
- name: Run Benchmarks
  run: cargo bench claude_code_performance_tests
```

## Troubleshooting

### Database Errors
Tests use in-memory SQLite databases. If you see migration errors:
```bash
cargo clean
cargo build
```

### Async Test Issues
All async tests use `#[tokio::test]` attribute and run in isolated runtimes.

### Mock Not Working
Check that the mock executor is properly configured:
```rust
let mut mock = MockCommandExecutor::new();
mock.mock_claude_success();  // Add expected responses
mock.mock_gh_pr_create();
```

## Test Coverage

Monitor coverage with:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --tests claude_code
```