#!/bin/bash

echo "🧪 Running Kanban TDD Tests"
echo "=========================="

# Start the Dioxus dev server in background
echo "Starting Dioxus server..."
dx serve --platform web --port 8080 &
SERVER_PID=$!

# Wait for server to start (longer wait for build)
echo "Waiting for server to build and start..."
sleep 60

# Check if server is running
if curl -s http://localhost:8080 > /dev/null; then
    echo "Server is running!"
else
    echo "Warning: Server might not be fully started yet"
fi

# Run the TDD tests
echo "Running tests..."
npx playwright test e2e-tests/kanban-tdd.spec.ts --reporter=list --project=chromium

# Kill the server
kill $SERVER_PID 2>/dev/null

echo "Tests complete!"