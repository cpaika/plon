# Claude Code Integration Test

## How to Test the Claude Button Feature

1. **Start the application:**
   ```bash
   cargo run --bin plon-desktop
   ```

2. **Navigate to Map View:**
   - Click on the "🗺️ Map" tab in the navigation bar

3. **View the Claude buttons:**
   - Each task card now has a green play button (▶) in the top-right corner
   - Tasks that are in progress show an orange lightning bolt (⚡) instead

4. **Launch Claude Code:**
   - Click the play button on any task
   - This will attempt to:
     - Create a branch for the task
     - Generate a TODO_CLAUDE.md file with task details
     - Launch Claude Code (or VS Code as fallback)
     - Update the task status to "InProgress"

5. **What happens when clicked:**
   - If Claude CLI is installed: Opens Claude Code with the task details
   - If not: Creates TODO_CLAUDE.md and tries to open VS Code
   - The task status changes to "InProgress" and button becomes orange

## Implementation Details

### Files Modified:
- `src/services/claude_automation.rs` - Claude automation service
- `src/ui_dioxus/views/map_final.rs` - Added play button to task cards
- `src/services/mod.rs` - Exported the new service

### Features:
- ✅ Play button on each task (top-right corner)
- ✅ Status indicator (green play = todo, orange lightning = in progress)
- ✅ Creates task-specific branch
- ✅ Generates task instructions for Claude
- ✅ Fallback to VS Code if Claude CLI not available
- ✅ Updates task status in UI when launched

### Future Enhancements:
- Configure repository URL from settings
- Track PR creation and link to task
- Show Claude's progress in real-time
- Add stop/cancel button for in-progress tasks
- Integration with GitHub PR workflow