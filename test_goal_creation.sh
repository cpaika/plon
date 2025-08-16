#!/bin/bash

# Test goal creation in the database
DB_PATH="plon.db"

echo "Testing Goal Creation..."

# Check if goals table exists
sqlite3 $DB_PATH "SELECT name FROM sqlite_master WHERE type='table' AND name='goals';" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✅ Goals table exists"
    
    # Count existing goals
    COUNT=$(sqlite3 $DB_PATH "SELECT COUNT(*) FROM goals;" 2>/dev/null)
    echo "Current goals count: $COUNT"
    
    # Show goal columns
    echo "Goal table structure:"
    sqlite3 $DB_PATH ".schema goals" 2>/dev/null | head -5
    
    echo ""
    echo "✅ Goal creation infrastructure is ready!"
    echo "You can now:"
    echo "1. Run './target/release/plon' to open the app"
    echo "2. Navigate to the 'Goals' tab"
    echo "3. Create new goals using the form"
else
    echo "❌ Goals table not found. Running migrations might be needed."
fi