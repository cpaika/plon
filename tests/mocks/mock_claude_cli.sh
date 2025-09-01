#!/bin/bash

# Mock Claude Code CLI for testing
# This script simulates the Claude Code CLI behavior for testing purposes

# Parse command line arguments
MODE="success"
DELAY=1
OUTPUT_FILE=""
INSTRUCTIONS_FILE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        code)
            shift
            ;;
        --file)
            shift
            OUTPUT_FILE="$1"
            shift
            ;;
        --instructions)
            shift
            INSTRUCTIONS_FILE="$1"
            shift
            ;;
        --mode)
            shift
            MODE="$1"
            shift
            ;;
        --delay)
            shift
            DELAY="$1"
            shift
            ;;
        *)
            shift
            ;;
    esac
done

# Log the invocation
echo "[$(date +'%Y-%m-%d %H:%M:%S')] Mock Claude Code CLI invoked" >&2
echo "Mode: $MODE" >&2
echo "Output file: $OUTPUT_FILE" >&2
echo "Instructions file: $INSTRUCTIONS_FILE" >&2

# Simulate different scenarios based on mode
case $MODE in
    success)
        echo "Starting Claude Code session..." >&2
        sleep $DELAY
        echo "Analyzing task requirements..." >&2
        sleep $DELAY
        echo "Implementing solution..." >&2
        
        # Create some mock changes
        if [ -f "$OUTPUT_FILE" ]; then
            echo "// Mock implementation by Claude Code" > mock_implementation.js
            echo "function solveTask() {" >> mock_implementation.js
            echo "    return 'Task completed successfully';" >> mock_implementation.js
            echo "}" >> mock_implementation.js
        fi
        
        sleep $DELAY
        echo "Running tests..." >&2
        sleep $DELAY
        echo "All tests passed!" >&2
        echo "Task completed successfully" >&2
        exit 0
        ;;
        
    partial)
        echo "Starting Claude Code session..." >&2
        sleep $DELAY
        echo "Analyzing task requirements..." >&2
        sleep $DELAY
        echo "Implementing solution..." >&2
        sleep $DELAY
        echo "Warning: Some tests are failing" >&2
        echo "Partial implementation complete" >&2
        exit 0
        ;;
        
    error)
        echo "Starting Claude Code session..." >&2
        sleep $DELAY
        echo "Error: Failed to understand task requirements" >&2
        echo "Please provide more detailed instructions" >&2
        exit 1
        ;;
        
    timeout)
        echo "Starting Claude Code session..." >&2
        sleep 100  # Simulate a long-running task that will timeout
        echo "This should never be printed" >&2
        exit 0
        ;;
        
    crash)
        echo "Starting Claude Code session..." >&2
        sleep $DELAY
        echo "Fatal error: Segmentation fault" >&2
        exit 139
        ;;
        
    *)
        echo "Unknown mode: $MODE" >&2
        exit 1
        ;;
esac