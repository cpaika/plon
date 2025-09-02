#!/bin/bash
cd '/Users/cpaika/.plon_claude_workspace/task_monitor-claude-status'
echo '🤖 Starting Claude for task: Monitor Claude Status'
echo ''
echo '📁 Directory: /Users/cpaika/.plon_claude_workspace/task_monitor-claude-status'
echo '🌿 Branch: task/79302f85-monitor-claude-status'
echo ''
echo 'Git Status:'
git status --short
echo ''
echo 'Starting Claude...'
echo ''
'/Users/cpaika/.claude/local/claude' --dangerously-skip-permissions '/Users/cpaika/.plon_claude_workspace/task_monitor-claude-status/task_prompt.md'
echo ''
echo 'Claude session completed. Press any key to close this window...'
read -n 1