#!/bin/bash

echo "=== CPU USAGE MONITOR FOR PLON-DESKTOP ==="
echo
echo "Starting monitoring... (Press Ctrl+C to stop)"
echo

# Get the PID of plon-desktop
PID=$(pgrep -f "plon-desktop")

if [ -z "$PID" ]; then
    echo "❌ plon-desktop is not running!"
    echo "Please start the app first with: cargo run --bin plon-desktop"
    exit 1
fi

echo "Found plon-desktop with PID: $PID"
echo
echo "Time     | CPU % | Status"
echo "---------|-------|----------------"

while true; do
    # Get CPU usage for the process
    CPU=$(ps -p $PID -o %cpu= 2>/dev/null | xargs)
    
    if [ -z "$CPU" ]; then
        echo "Process ended"
        break
    fi
    
    # Get current time
    TIME=$(date +%H:%M:%S)
    
    # Determine status based on CPU usage
    STATUS="✅ Normal"
    if (( $(echo "$CPU > 80" | bc -l) )); then
        STATUS="⚠️  HIGH!"
    elif (( $(echo "$CPU > 50" | bc -l) )); then
        STATUS="⚡ Active"
    fi
    
    printf "%s | %5.1f%% | %s\n" "$TIME" "$CPU" "$STATUS"
    
    sleep 1
done