# Testing Guide for Plon

This document describes the comprehensive testing framework for the Plon task management application.

## Test Types

### 1. Integration Tests (`tests/test_integration.rs`)
These tests verify the database and repository layers work correctly.

**Run all integration tests:**
```bash
cargo test --test test_integration
```

**Available tests:**
- `test_task_persistence` - Verifies tasks save and load from database
- `test_dependency_persistence` - Confirms dependencies persist correctly  
- `test_task_position_update` - Tests position updates are saved
- `test_duplicate_dependency_prevention` - Ensures no duplicate dependencies

### 2. Component Tests (`tests/test_dioxus_components.rs`)
Tests individual Dioxus components in isolation.

**Run component tests:**
```bash
cargo test --test test_dioxus_components
```

**Test coverage:**
- Component rendering
- State management
- Dependency management
- Drag state handling
- Database integration

### 3. End-to-End UI Tests (`tests/test_e2e_enigo.rs`)
Automated UI tests using enigo to control mouse and keyboard.

**Run all E2E tests (requires display):**
```bash
cargo test --test test_e2e_enigo -- --ignored --nocapture
```

**Run specific E2E test:**
```bash
# Test task creation
cargo test test_create_task_via_ui -- --ignored --nocapture

# Test dependency creation via drag-and-drop
cargo test test_create_dependency_via_drag -- --ignored --nocapture

# Test persistence across restarts
cargo test test_task_persistence -- --ignored --nocapture

# Test Kanban drag-and-drop
cargo test test_kanban_drag_drop -- --ignored --nocapture

# Test map zoom and pan
cargo test test_map_view_zoom_pan -- --ignored --nocapture

# Full workflow test
cargo test test_full_workflow -- --ignored --nocapture
```

## Prerequisites

### For E2E Tests
- Display required (won't work in headless environments)
- macOS: Accessibility permissions may be required for enigo
- Linux: X11 or Wayland display
- Windows: No special requirements

### Screenshots
E2E tests capture screenshots in `test-screenshots/` directory:
```bash
mkdir -p test-screenshots
```

## Running Tests in CI/CD

### GitHub Actions Example
```yaml
name: Tests

on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run integration tests
        run: cargo test --test test_integration

  component-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run component tests
        run: cargo test --test test_dioxus_components

  e2e-tests:
    runs-on: macos-latest  # Or ubuntu-latest with xvfb
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Create screenshot directory
        run: mkdir -p test-screenshots
      - name: Run E2E tests
        run: |
          # On Linux, you might need: xvfb-run cargo test ...
          cargo test --test test_e2e_enigo -- --ignored --nocapture
      - name: Upload screenshots
        if: always()
        uses: actions/upload-artifact@v2
        with:
          name: test-screenshots
          path: test-screenshots/
```

## Test Coverage Areas

### Map View
- ✅ Task creation and positioning
- ✅ Dependency creation via drag-and-drop (right node → left node)
- ✅ Task movement and position persistence
- ✅ Zoom and pan controls
- ✅ Database persistence

### Kanban View
- ✅ Task creation in columns
- ✅ Drag tasks between columns
- ✅ Status updates persist
- ✅ Column ordering

### Data Persistence
- ✅ Tasks persist across app restarts
- ✅ Dependencies persist to SQLite database
- ✅ Position updates are saved
- ✅ Duplicate dependencies prevented

## Debugging Failed Tests

### Integration Test Failures
1. Check database migrations are up to date
2. Verify SQLite is available
3. Check for file permission issues

### E2E Test Failures
1. Screenshots are saved in `test-screenshots/`
2. Add more `thread::sleep()` calls if timing issues occur
3. Adjust coordinates if UI layout changes
4. Check accessibility permissions (macOS)

### Component Test Failures
1. Ensure Dioxus version compatibility
2. Check for breaking changes in component APIs
3. Verify state management logic

## Performance Testing

Run benchmarks:
```bash
cargo bench
```

## Test Database

Tests use an in-memory SQLite database via `init_test_database()` to ensure isolation.

## Writing New Tests

### Integration Test Template
```rust
#[tokio::test]
async fn test_new_feature() {
    let pool = init_test_database().await.unwrap();
    let repo = Repository::new(pool);
    
    // Test logic here
    
    assert!(condition);
}
```

### E2E Test Template
```rust
#[test]
#[ignore] // Run with --ignored flag
fn test_ui_feature() {
    let mut app = AppHandle::new();
    
    // UI interactions
    app.click_at(x, y);
    app.drag_from_to(x1, y1, x2, y2);
    
    // Capture result
    app.take_screenshot("test_result");
}
```

## Continuous Testing

For development, run tests in watch mode:
```bash
cargo watch -x "test --test test_integration"
```

## Test Maintenance

- Update coordinates in E2E tests when UI layout changes
- Keep integration tests fast by using in-memory databases
- Screenshot comparisons may need tolerance adjustments
- Document any flaky tests and mitigation strategies