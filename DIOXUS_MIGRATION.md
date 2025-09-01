# Dioxus Migration Guide

## Overview
This project has been migrated from egui to Dioxus, a reactive UI framework for Rust. Along with the UI migration, the testing framework has been updated to use Playwright for end-to-end testing.

## Key Changes

### UI Framework: egui → Dioxus
- **egui**: Immediate mode GUI, desktop-focused
- **Dioxus**: React-like reactive framework with web and desktop support
- Component-based architecture with hooks and state management
- Support for both web (WASM) and desktop targets

### State Management
- Using Fermi for global state management (similar to Redux/Recoil)
- Atoms for shared state across components
- Signals for local component state

### Testing: Custom Tests → Playwright
- Migrated from Rust-based UI tests to Playwright E2E tests
- Cross-browser testing support (Chromium, Firefox, WebKit)
- Better debugging tools and test reporting

## Project Structure

```
src/
├── ui_dioxus/          # New Dioxus UI implementation
│   ├── app.rs          # Main app component and routing
│   ├── state.rs        # Global state management with Fermi
│   ├── router.rs       # Route definitions
│   └── views/          # View components
│       ├── map_view.rs     # Interactive task map
│       ├── list_view.rs    # Task list with filtering
│       ├── kanban_view.rs  # Kanban board with drag-drop
│       ├── timeline_view.rs # Timeline/calendar view
│       └── gantt_view.rs   # Gantt chart view
├── bin/
│   ├── plon-desktop.rs # Desktop app entry point
│   └── plon-web.rs     # Web app entry point
e2e-tests/              # Playwright E2E tests
├── map-view.spec.ts
├── list-view.spec.ts
├── kanban-view.spec.ts
└── timeline-view.spec.ts
```

## Running the Application

### Desktop Mode
```bash
cargo run --bin plon-desktop
```

### Web Mode
```bash
cargo run --bin plon-web
# Opens at http://localhost:8080
```

## Testing

### Setup Playwright
```bash
./setup-playwright.sh
```

### Run E2E Tests
```bash
# Run all tests
npm test

# Run specific browser
npm run test:chromium
npm run test:firefox
npm run test:webkit

# Run with UI
npm run test:headed

# Debug tests
npm run test:debug
```

## Features Implemented

### Views
1. **Map View**: SVG-based task visualization with drag-and-drop
2. **List View**: Filterable/sortable task list with inline editing
3. **Kanban View**: Drag-and-drop between status columns
4. **Timeline View**: Calendar-based task scheduling
5. **Gantt View**: Project timeline with dependencies

### Interactions
- Task selection and editing
- Drag-and-drop support
- Play button for Claude Code execution
- Real-time status updates
- Zoom and pan controls

## Migration Status

✅ **Completed**:
- Dioxus app structure and routing
- All main views ported to Dioxus
- State management with Fermi
- Playwright test setup
- Example E2E tests for each view
- Web and desktop binaries

🚧 **Todo**:
- Complete service integration
- Port remaining UI tests to Playwright
- Add CSS styling
- Implement task persistence
- Add WebSocket support for real-time updates

## Development Tips

### Adding New Components
1. Create component in `src/ui_dioxus/views/`
2. Add route in `src/ui_dioxus/router.rs`
3. Export from `src/ui_dioxus/views/mod.rs`
4. Create Playwright test in `e2e-tests/`

### State Management
- Use Fermi atoms for global state
- Use signals for local component state
- Keep state updates immutable

### Testing Best Practices
- Use data-testid attributes for reliable element selection
- Test user interactions, not implementation details
- Keep tests independent and idempotent
- Use Playwright's built-in wait strategies

## Benefits of Migration

1. **Cross-platform**: Single codebase for web and desktop
2. **Modern Architecture**: Component-based, reactive UI
3. **Better Testing**: Playwright provides superior E2E testing
4. **Performance**: Virtual DOM and efficient rendering
5. **Developer Experience**: Hot reload, better debugging tools
6. **Future-proof**: Active development and growing ecosystem