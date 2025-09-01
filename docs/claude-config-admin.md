# Claude Code Configuration Admin Page

## Overview
The Claude Code Configuration admin page allows you to configure how Plon integrates with Claude Code for automated task execution.

## Accessing the Admin Page

1. Launch the Plon application:
   ```bash
   cargo run
   ```

2. Navigate to the Claude Config page:
   - Click the "⚙️ Claude Config" button in the navigation menu
   - Or directly navigate to: `/admin/claude-config`

## Configuration Settings

### GitHub Settings
- **Repository Owner**: Your GitHub username or organization (e.g., "cpaika")
- **Repository Name**: The name of your repository (e.g., "plon")
- **Default Base Branch**: The branch to base new feature branches off of (default: "main")
- **GitHub Token**: Optional - only needed for private repositories (starts with `ghp_`)

### Workspace Settings
- **Workspace Root Directory**: Where task workspaces will be created
  - Default: `~/plon-workspaces`
  - Each task creates a subfolder like: `task-[id]-[title]`
- **Custom Git Clone URL**: Optional - overrides the default GitHub URL
  - If not set, uses: `https://github.com/{owner}/{repo}.git`

### Claude Settings
- **Claude API Key**: Required for Claude Code to function (starts with `sk-ant-`)
- **Claude Model**: Select which Claude model to use
  - Claude 3 Opus (most capable)
  - Claude 3 Sonnet (balanced)
  - Claude 3 Haiku (fastest)
- **Max Session Duration**: How long Claude Code can run (5-240 minutes)
- **Auto Create PR**: Whether to automatically create pull requests after task completion

## How It Works

When you execute a task with Claude Code:

1. **Workspace Creation**: A new folder is created in your workspace root directory
   - Named: `task-{first-8-chars-of-id}-{slugified-title}`
   - Example: `task-a1b2c3d4-fix-authentication-bug`

2. **Repository Cloning**: The configured repository is cloned into the task workspace
   - Uses the custom Git URL if provided
   - Otherwise uses: `https://github.com/{owner}/{repo}.git`

3. **Branch Creation**: A new feature branch is created
   - Format: `claude/{task-id}-{task-title}`

4. **Claude Code Execution**: Claude Code runs in the isolated workspace
   - Uses the provided API key and model
   - Works on the task independently

5. **Pull Request**: If configured, automatically creates a PR when complete

## Benefits of Isolated Workspaces

- **No Conflicts**: Each task has its own clean workspace
- **Parallel Execution**: Multiple tasks can run simultaneously
- **Clean State**: Every task starts with a fresh clone of the repository
- **Easy Cleanup**: Task folders can be deleted when no longer needed
- **Debugging**: Each task's work is preserved in its own folder

## Example Configuration

For a typical setup:
- Repository Owner: `your-username`
- Repository Name: `your-project`
- Workspace Root: `/home/user/claude-workspaces`
- Claude API Key: `sk-ant-...`
- Model: Claude 3 Opus
- Max Duration: 60 minutes
- Auto Create PR: ✓ Enabled

This would create workspaces like:
- `/home/user/claude-workspaces/task-abc12345-implement-feature/`
- `/home/user/claude-workspaces/task-def67890-fix-bug/`

Each containing a fresh clone of `https://github.com/your-username/your-project.git`