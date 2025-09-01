#!/bin/bash

echo "Testing Claude Code error handling..."

# Test 1: Check if Claude is installed
echo "Test 1: Checking if Claude CLI is available..."
if which claude > /dev/null 2>&1; then
    echo "✅ Claude CLI found at: $(which claude)"
    claude --version
else
    echo "❌ Claude CLI not found - the app will show an error message"
fi

echo ""

# Test 2: Check if we're in a git repository
echo "Test 2: Checking git repository..."
if git rev-parse --git-dir > /dev/null 2>&1; then
    echo "✅ Git repository found"
    echo "Current branch: $(git branch --show-current)"
else
    echo "❌ Not in a git repository - the app will show an error message"
fi

echo ""

# Test 3: Try the actual Claude command format
echo "Test 3: Testing Claude command format..."
echo "The app will run: claude code --task-file <file> --auto-pr --pr-title <title>"

# Create a test task file
TEST_FILE=".test_claude_task.md"
cat > $TEST_FILE << EOF
# Test Task

This is a test task to verify Claude Code integration.
EOF

if which claude > /dev/null 2>&1; then
    echo "Attempting to run Claude Code with test task..."
    claude code --help 2>/dev/null | grep -q "task-file" && echo "✅ Claude supports --task-file option" || echo "⚠️  Claude may not support --task-file option"
else
    echo "⚠️  Cannot test Claude command - CLI not installed"
fi

# Clean up
rm -f $TEST_FILE

echo ""
echo "Test complete. The app will show detailed error messages if:"
echo "1. Claude CLI is not installed"
echo "2. Not in a git repository"
echo "3. Claude command fails to execute"