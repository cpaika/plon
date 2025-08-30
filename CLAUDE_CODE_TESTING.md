# Claude Code Integration Testing Guide

## Overview

The Claude Code integration includes a comprehensive test suite with mocked CLI tools to ensure reliable functionality without requiring actual Claude Code or GitHub CLI installations during testing.

## Test Architecture

### 1. Mock Infrastructure

#### Mock Claude CLI (`tests/mocks/mock_claude_cli.sh`)
Simulates Claude Code CLI with configurable behaviors:
- **success**: Normal successful execution with PR creation
- **partial**: Partial completion with warnings
- **error**: Failure with clear error message
- **timeout**: Long-running task that times out
- **crash**: Simulates unexpected crash

#### Mock GitHub CLI (`tests/mocks/mock_gh_cli.sh`)
Simulates `gh` CLI for PR creation:
- Creates mock PR URLs with random PR numbers
- Simulates authentication status checks
- Returns realistic GitHub URLs

### 2. Test Utilities (`tests/claude_code_test_utils.rs`)

#### ClaudeCodeTestEnvironment
Complete test environment with:
- Temporary directory with git repository
- In-memory SQLite database
- Mock CLI path configuration
- Pre-configured Claude Code settings

#### MockClaudeCodeService
In-memory service mock for unit testing:
- Configurable failure modes
- Adjustable execution delays
- Session state tracking

#### Test Fixtures
Pre-configured test data:
- `sample_task()`: Standard task with metadata
- `complex_task()`: Multi-line task with detailed requirements
- `minimal_task()`: Simple task for edge cases
- `sample_config()`: Default Claude Code configuration
- `sample_template()`: Standard prompt template

## Test Categories

### 1. Unit Tests

**Domain Model Tests**
- Session creation and state management
- Status transitions and validation
- Configuration validation
- Template variable extraction and rendering

**Service Layer Tests**
- Task launching logic
- Git branch creation
- PR creation workflow
- Session cancellation

**Repository Tests**
- CRUD operations for sessions
- Configuration persistence
- Template management
- Session querying and filtering

### 2. Integration Tests (`tests/claude_code_integration_tests.rs`)

**Core Workflows**
- `test_launch_claude_code_success`: Full successful workflow
- `test_launch_claude_code_with_error`: Error handling
- `test_multiple_concurrent_sessions`: Concurrent execution
- `test_pr_creation_flow`: GitHub PR creation

**Session Management**
- `test_session_cancellation`: Cancel running sessions
- `test_session_timeout_detection`: Timeout handling
- `test_session_persistence_and_retrieval`: Database operations
- `test_session_log_accumulation`: Log management

**Configuration**
- `test_configuration_management`: Config CRUD operations
- `test_template_management`: Template handling
- `test_cleanup_old_sessions`: Automated cleanup

### 3. UI Tests (`tests/claude_code_ui_tests.rs`)

**Component Tests**
- `test_claude_code_button_in_task_editor`: Launch button presence
- `test_claude_code_view_navigation`: View navigation
- `test_session_status_colors`: Status indicators
- `test_active_session_indicator`: Running state display

**Interaction Tests**
- `test_cancel_button_for_active_sessions`: Cancellation UI
- `test_retry_button_for_failed_sessions`: Retry functionality
- `test_empty_task_description_warning`: User guidance

### 4. Performance Tests (`tests/claude_code_performance_tests.rs`)

**Benchmarks**
- Session creation performance
- Status update overhead
- Log accumulation with large datasets
- Template rendering speed
- Database operation latency
- Concurrent session management
- Session filtering performance

## Running Tests

### Quick Test
```bash
# Run all Claude Code tests
cargo test claude_code

# Run specific test category
cargo test --test claude_code_integration_tests
cargo test --test claude_code_ui_tests
```

### Comprehensive Test Suite
```bash
# Run the complete test suite with detailed output
./run_claude_code_tests.sh

# Include stress tests
./run_claude_code_tests.sh --stress
```

### Performance Benchmarks
```bash
# Run performance benchmarks
cargo bench claude_code_performance_tests

# Generate detailed benchmark report
cargo bench claude_code_performance_tests -- --verbose
```

### Individual Test Execution
```bash
# Run specific test
cargo test test_launch_claude_code_success -- --nocapture

# Run with logging
RUST_LOG=debug cargo test test_session_monitoring
```

## Writing New Tests

### 1. Integration Test Template
```rust
#[tokio::test]
async fn test_new_feature() -> Result<()> {
    // Setup test environment
    let mut env = ClaudeCodeTestEnvironment::new().await?;
    
    // Create test data
    let task = fixtures::sample_task();
    env.repository.tasks.create(&task).await?;
    
    // Execute test scenario
    let session = env.launch_with_mock(&task, "success").await?;
    
    // Verify results
    assert_eq!(session.status, SessionStatus::Working);
    
    // Cleanup
    env.cleanup().await?;
    Ok(())
}
```

### 2. UI Test Template
```rust
#[test]
fn test_ui_component() {
    let mut app = PlonApp::new_for_test();
    
    // Setup test state
    app.current_view = ViewType::ClaudeCode;
    
    // Create test harness
    let mut harness = Harness::new(move |ctx| {
        app.update(ctx, &mut eframe::Frame::default());
    });
    
    harness.run();
    
    // Verify UI elements
    let element = harness.get_by_label("Expected Label");
    assert!(element.is_some());
}
```

### 3. Mock Behavior Configuration
```rust
// Configure mock for specific test scenario
let session = env.launch_with_mock(&task, "error").await?;

// Available modes:
// - "success": Normal completion
// - "error": Controlled failure
// - "timeout": Long-running timeout
// - "partial": Partial completion
// - "crash": Unexpected failure
```

## Test Data Management

### Database Setup
Tests use in-memory SQLite databases that are automatically:
- Created fresh for each test
- Migrated with latest schema
- Cleaned up after test completion

### Temporary Files
Test environment creates temporary directories for:
- Git repositories
- Claude Code working directories
- Test artifacts

All cleaned up automatically via Rust's `TempDir`.

### Mock Process Management
Mock CLI scripts are managed through:
- PATH manipulation for test isolation
- Environment variables for behavior control
- Process cleanup on test completion

## Debugging Tests

### Enable Detailed Logging
```bash
# Set logging level
RUST_LOG=debug cargo test test_name -- --nocapture

# Log specific modules
RUST_LOG=plon::services::claude_code_service=trace cargo test
```

### Inspect Test Artifacts
```bash
# Keep temporary directories for inspection
KEEP_TEST_DIRS=1 cargo test test_name

# Check mock script output
./tests/mocks/mock_claude_cli.sh code --mode success
```

### Database Inspection
```rust
// Add debug prints in tests
println!("Sessions: {:?}", repository.claude_code.get_active_sessions().await?);

// Use SQL logging
RUST_LOG=sqlx=debug cargo test
```

## CI/CD Integration

### GitHub Actions Configuration
```yaml
- name: Run Claude Code Tests
  run: |
    ./run_claude_code_tests.sh
    
- name: Run Performance Benchmarks
  run: |
    cargo bench claude_code_performance_tests --no-fail-fast
```

### Pre-commit Hook
```bash
#!/bin/bash
# .git/hooks/pre-commit

# Run quick tests before commit
cargo test claude_code --lib --bins
```

## Common Issues and Solutions

### Mock Scripts Not Found
**Problem**: Tests fail with "command not found"
**Solution**: Ensure mock scripts are executable:
```bash
chmod +x tests/mocks/*.sh
```

### Database Migration Errors
**Problem**: Schema mismatch errors
**Solution**: Update migrations and regenerate:
```bash
sqlx migrate run
cargo clean
cargo build
```

### Timeout in Tests
**Problem**: Tests timeout waiting for completion
**Solution**: Increase timeout or check mock script delays:
```rust
env.wait_for_completion(session.id, 30).await? // Increase timeout
```

### Flaky Concurrent Tests
**Problem**: Race conditions in concurrent tests
**Solution**: Use proper synchronization:
```rust
tokio::time::sleep(Duration::from_millis(100)).await;
```

## Test Coverage

Monitor test coverage with:
```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

Target coverage goals:
- Domain models: 90%+
- Service layer: 85%+
- Repository layer: 80%+
- UI components: 70%+

## Best Practices

1. **Isolation**: Each test should be independent
2. **Cleanup**: Always clean up resources
3. **Mocking**: Use mocks for external dependencies
4. **Assertions**: Be specific in assertions
5. **Documentation**: Document complex test scenarios
6. **Performance**: Keep tests fast (< 1 second each)
7. **Determinism**: Avoid random data when possible
8. **Error Cases**: Test both success and failure paths

## Contributing Tests

When adding new Claude Code features:
1. Write unit tests for new domain models
2. Add integration tests for workflows
3. Create UI tests for new components
4. Include performance benchmarks if applicable
5. Update test documentation
6. Ensure all tests pass before PR submission