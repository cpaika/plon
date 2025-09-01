# Accessing Settings in Plon

## How to Open Settings

1. **Launch the Application**
   ```bash
   cargo run
   ```

2. **Navigate to Settings**
   - Look for the **"⚙️ Settings"** button in the navigation menu
   - It's located after the main view tabs (Map, List, Kanban, Timeline, Gantt)
   - Click on it to open the Settings page

## Settings Page Layout

The Settings page is organized into multiple tabs:

### 🤖 Claude Code Tab (Default)
- **Purpose**: Configure Claude AI integration for automated task execution
- **Contents**:
  - GitHub repository settings
  - Workspace directory configuration
  - Claude API key and model selection
  - Auto-PR creation toggle
- **Key Settings**:
  - Repository owner and name
  - Workspace root (default: `~/plon-workspaces`)
  - Claude API key (required for AI features)

### ⚙️ General Tab
- **Purpose**: General application preferences
- **Status**: Coming soon
- **Planned Features**:
  - Default task settings
  - Notification preferences
  - Data management options

### 📁 Workspace Tab
- **Purpose**: File and directory management
- **Status**: Coming soon
- **Planned Features**:
  - Default project paths
  - Backup settings
  - File organization rules

### 🔗 Integrations Tab
- **Purpose**: External service connections
- **Current Integrations**:
  - GitHub (for repository management)
  - Claude AI (configured in Claude Code tab)
  - Slack (coming soon)
- **Shows**: Connection status for each integration

### 🎨 Appearance Tab
- **Purpose**: UI customization
- **Features**:
  - Theme selection (Light/Dark/Auto)
  - Accent color customization
- **Status**: UI preview only (functionality coming soon)

## Quick Access Path

```
Application Launch → Navigation Bar → ⚙️ Settings → Select Tab
```

## Most Common Settings Task

To configure Claude Code for automated task execution:
1. Click **⚙️ Settings** in navigation
2. The **Claude Code** tab is selected by default
3. Fill in:
   - GitHub repository details
   - Claude API key
   - Workspace preferences
4. Click **Save Configuration**

The settings are now accessible through a dedicated Settings tab in the main navigation, making it easy to find and configure all application preferences in one place.