-- Claude Code Integration Schema
-- This migration adds support for launching Claude Code instances from tasks

-- Table to track Claude Code sessions
CREATE TABLE IF NOT EXISTS claude_code_sessions (
    id TEXT PRIMARY KEY NOT NULL,
    task_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('pending', 'initializing', 'working', 'creating_pr', 'completed', 'failed', 'cancelled')),
    branch_name TEXT,
    pr_url TEXT,
    pr_number INTEGER,
    session_log TEXT,
    error_message TEXT,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);

-- Configuration table for Claude Code and GitHub settings
CREATE TABLE IF NOT EXISTS claude_code_config (
    id TEXT PRIMARY KEY NOT NULL,
    github_repo TEXT NOT NULL,
    github_owner TEXT NOT NULL,
    github_token TEXT, -- Will be encrypted in production
    claude_api_key TEXT, -- Will be encrypted in production
    default_base_branch TEXT DEFAULT 'main',
    auto_create_pr BOOLEAN DEFAULT true,
    working_directory TEXT,
    claude_model TEXT DEFAULT 'claude-3-opus-20240229',
    max_session_duration_minutes INTEGER DEFAULT 60,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Table to store Claude Code prompt templates
CREATE TABLE IF NOT EXISTS claude_prompt_templates (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    template TEXT NOT NULL,
    description TEXT,
    variables TEXT, -- JSON array of variable names
    is_default BOOLEAN DEFAULT false,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default prompt template
INSERT INTO claude_prompt_templates (id, name, template, description, variables, is_default)
VALUES (
    '00000000-0000-0000-0000-000000000001',
    'default',
    'You are an AI assistant helping with the following task:

Task Title: {{task_title}}
Task Description:
{{task_description}}

Additional Context:
- Parent Goal: {{goal_title}}
- Priority: {{priority}}
- Estimated Hours: {{estimated_hours}}
- Tags: {{tags}}

Instructions:
1. Analyze the task requirements
2. Implement the necessary changes
3. Write tests if applicable
4. Ensure code quality and documentation
5. Create a pull request with a clear description

When you''re done, create a PR with:
- Clear title describing the changes
- Detailed description of what was implemented
- Any relevant testing information
- Notes about design decisions

Branch naming convention: feature/{{task_id_short}}-{{task_title_slug}}',
    'Default template for Claude Code task execution',
    '["task_title", "task_description", "goal_title", "priority", "estimated_hours", "tags", "task_id_short", "task_title_slug"]',
    true
);

-- Add indexes for performance
CREATE INDEX idx_claude_sessions_task_id ON claude_code_sessions(task_id);
CREATE INDEX idx_claude_sessions_status ON claude_code_sessions(status);
CREATE INDEX idx_claude_sessions_started_at ON claude_code_sessions(started_at);