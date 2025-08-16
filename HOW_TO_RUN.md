# How to Run Plon

## Quick Start

There is now only ONE way to run the app:

```bash
cargo run
```

That's it! This will:
1. Build the application if needed
2. Initialize the database
3. Launch the GUI with the full-featured Kanban board (with drag-and-drop!)

## Features Available

- **List View** - Traditional task list
- **Kanban Board** - Full drag-and-drop support, WIP limits, quick add
- **Map View** - Spatial task organization  
- **Timeline** - Calendar and schedule views
- **Gantt Chart** - Project timeline visualization
- **Dashboard** - Overview and analytics
- **Goals** - Goal tracking and management
- **Resources** - Resource management
- **Recurring Tasks** - Template-based recurring tasks

## Kanban Board Features

The Kanban board now includes:
- ✅ **Drag & Drop** - Click and drag cards between columns
- ✅ **WIP Limits** - Column work-in-progress limits
- ✅ **Quick Add** - Click ➕ to quickly add tasks to any column
- ✅ **Search** - Filter tasks by text
- ✅ **Multi-select** - Select multiple cards for bulk operations
- ✅ **Keyboard shortcuts** - Arrow keys to move selected tasks
- ✅ **Priority colors** - Visual indicators for task priority
- ✅ **Collapsible columns** - Click ▼/▶ to collapse/expand columns

## Database

The app uses SQLite and will automatically create `plon.db` in the current directory on first run.

## Troubleshooting

If you have any issues:
1. Delete `plon.db` to start fresh
2. Run `cargo clean` and then `cargo run` again
3. Make sure you have Rust installed (https://rustup.rs/)

## Development

To run tests:
```bash
cargo test
```

To run specific Kanban tests:
```bash
cargo test kanban
```