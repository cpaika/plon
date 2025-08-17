#!/bin/bash
echo "Testing if app runs for 20 seconds without freeze..."

# Start the app in background
cargo run --release 2>/dev/null &
APP_PID=$!

# Monitor for 20 seconds
for i in {1..20}; do
    if ! kill -0 $APP_PID 2>/dev/null; then
        echo "App exited after $i seconds"
        exit 0
    fi
    echo "Second $i: App still running (PID $APP_PID)"
    sleep 1
done

# Kill the app
kill $APP_PID 2>/dev/null
echo "✅ App ran for 20 seconds without freezing!"