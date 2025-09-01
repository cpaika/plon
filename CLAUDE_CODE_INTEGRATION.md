# Claude Code Integration for Plon

## Overview
Plon now includes integration with Claude Code, allowing you to launch AI-powered coding assistants directly from task views. Each Claude Code instance can work autonomously on a task and create a GitHub pull request when complete.

## Features

### 1. Launch Claude Code from Tasks
- Click the "🤖 Launch Claude Code" button in the task editor
- Claude Code will receive the task title, description, and metadata
- The AI assistant works autonomously on implementing the task

### 2. Session Management
- Track all Claude Code sessions in the dedicated "🤖 Claude Code" view
- Monitor session status: Pending, Initializing, Working, Creating PR, Completed, Failed
- View session logs and error messages
- Cancel running sessions if needed

### 3. GitHub Integration
- Automatic branch creation for each task
- Creates pull requests when work is complete
- Branch naming: `claude/<task-id-short>-<task-title-slug>`
- Links to PRs displayed in the UI

### 4. Configuration
- Set GitHub repository and owner
- Configure default base branch
- Set Claude API key (optional - can use environment variable)
- Configure working directory
- Adjust session timeout limits

## Setup Instructions

### 1. Database Migration
Run the migration to set up Claude Code tables:
```bash
sqlx migrate run
```

### 2. Configure GitHub Access
Ensure you have GitHub CLI installed and authenticated:
```bash
gh auth login
```

### 3. Configure Claude Code
1. Launch Plon: `cargo run`
2. Navigate to the "🤖 Claude Code" view
3. Click "Show Config"
4. Enter your configuration:
   - GitHub Owner: Your GitHub username or organization
   - GitHub Repo: Repository name
   - Default Branch: Usually "main" or "master"
   - Working Directory: Path to your project (optional)
   - Claude API Key: Your Anthropic API key (optional if set in environment)

### 4. Install Claude Code CLI
Ensure Claude Code CLI is installed and available in your PATH:
```bash
# Installation instructions depend on your platform
# Visit: https://claude.ai/code for installation guide
```

## Usage

### Launching Claude Code for a Task

1. **Create or Select a Task**
   - Open the task editor by clicking on a task
   - Ensure the task has a clear title and detailed description

2. **Launch Claude Code**
   - Click the "🤖 Launch Claude Code" button
   - Claude Code will start working on the task automatically

3. **Monitor Progress**
   - Go to the "🤖 Claude Code" view
   - Select your session to view logs
   - Watch as Claude Code works through the implementation

4. **Review the Pull Request**
   - Once complete, a PR link will appear
   - Click to review the changes on GitHub
   - Merge when satisfied with the implementation

### Session States

- **Pending**: Session created but not started
- **Initializing**: Setting up git branch and environment
- **Working**: Claude Code is actively working on the task
- **Creating PR**: Creating the pull request
- **Completed**: Successfully created PR
- **Failed**: Error occurred (check logs)
- **Cancelled**: Manually cancelled by user

## Prompt Templates

The system uses customizable prompt templates to guide Claude Code. The default template includes:
- Task title and description
- Priority and estimated hours
- Tags and metadata
- Instructions for implementation and PR creation

You can customize templates by modifying the `claude_prompt_templates` table.

## Architecture

### Components

1. **Domain Models** (`src/domain/claude_code.rs`)
   - `ClaudeCodeSession`: Tracks individual Claude Code sessions
   - `ClaudeCodeConfig`: Stores configuration settings
   - `ClaudePromptTemplate`: Manages prompt templates

2. **Service Layer** (`src/services/claude_code_service.rs`)
   - Orchestrates Claude Code process launching
   - Manages git operations
   - Handles PR creation via GitHub CLI

3. **Repository Layer** (`src/repository/claude_code_repository.rs`)
   - Database operations for sessions, config, and templates

4. **UI Components**
   - Task Editor: Launch button and session status
   - Claude Code View: Full session management interface

### Process Flow

1. User clicks "Launch Claude Code" for a task
2. System creates a new session record
3. Creates a new git branch
4. Generates prompt from template
5. Launches Claude Code CLI with task context
6. Claude Code works on implementation
7. Commits changes to branch
8. Creates PR via GitHub CLI
9. Updates session with PR information

## Security Considerations

- API keys are stored in the database (should be encrypted in production)
- GitHub tokens should use minimal required permissions
- Working directory access is restricted to configured path
- Session timeouts prevent runaway processes

## Troubleshooting

### Claude Code Not Launching
- Verify Claude Code CLI is installed: `which claude`
- Check API key is configured
- Ensure working directory exists and has git initialized

### PR Creation Failing
- Verify GitHub CLI is authenticated: `gh auth status`
- Check repository permissions
- Ensure base branch exists

### Sessions Timing Out
- Increase `max_session_duration_minutes` in configuration
- Check task complexity - break into smaller subtasks if needed

## Future Enhancements

- [ ] Real-time log streaming
- [ ] Multiple Claude Code models support
- [ ] Team collaboration features
- [ ] Cost tracking and limits
- [ ] Automated testing integration
- [ ] Custom code review workflows
- [ ] Integration with CI/CD pipelines

## Example Task Description

For best results, provide detailed task descriptions:

```markdown
# Task: Implement User Authentication

## Requirements
- Add login/logout endpoints
- Use JWT tokens for session management
- Implement password hashing with bcrypt
- Add user registration with email validation

## Technical Details
- Framework: Express.js
- Database: PostgreSQL
- Testing: Jest

## Acceptance Criteria
- [ ] Users can register with email/password
- [ ] Users can login and receive JWT token
- [ ] Protected routes require valid token
- [ ] Passwords are securely hashed
- [ ] Unit tests cover all endpoints
```

## Support

For issues or questions about the Claude Code integration:
1. Check the session logs in the Claude Code view
2. Review this documentation
3. Check the GitHub repository for known issues
4. Contact support with session ID and error details