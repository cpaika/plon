#!/bin/bash

# Mock GitHub CLI for testing
# This script simulates gh CLI behavior for testing purposes

# Parse command line arguments
COMMAND=""
SUBCOMMAND=""
PR_TITLE=""
PR_BODY=""
BASE_BRANCH=""
HEAD_BRANCH=""

while [[ $# -gt 0 ]]; do
    case $1 in
        pr)
            COMMAND="pr"
            shift
            ;;
        create)
            SUBCOMMAND="create"
            shift
            ;;
        --title)
            shift
            PR_TITLE="$1"
            shift
            ;;
        --body)
            shift
            PR_BODY="$1"
            shift
            ;;
        --base)
            shift
            BASE_BRANCH="$1"
            shift
            ;;
        --head)
            shift
            HEAD_BRANCH="$1"
            shift
            ;;
        auth)
            COMMAND="auth"
            shift
            ;;
        status)
            SUBCOMMAND="status"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Log the invocation
echo "[$(date +'%Y-%m-%d %H:%M:%S')] Mock gh CLI invoked" >&2
echo "Command: $COMMAND $SUBCOMMAND" >&2

# Handle different commands
if [ "$COMMAND" = "pr" ] && [ "$SUBCOMMAND" = "create" ]; then
    echo "Creating pull request..." >&2
    echo "Title: $PR_TITLE" >&2
    echo "Base: $BASE_BRANCH" >&2
    echo "Head: $HEAD_BRANCH" >&2
    
    # Simulate PR creation delay
    sleep 1
    
    # Return a mock PR URL
    PR_NUMBER=$((RANDOM % 1000 + 1))
    echo "https://github.com/test-owner/test-repo/pull/$PR_NUMBER"
    echo "Pull request created successfully" >&2
    exit 0
elif [ "$COMMAND" = "auth" ] && [ "$SUBCOMMAND" = "status" ]; then
    echo "github.com" 
    echo "  ✓ Logged in to github.com as test-user"
    echo "  ✓ Git operations for github.com configured to use https protocol."
    echo "  ✓ Token: *******************"
    exit 0
else
    echo "Mock gh: Unhandled command: $COMMAND $SUBCOMMAND" >&2
    exit 1
fi