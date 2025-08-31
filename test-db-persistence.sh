#!/bin/bash

echo "Database Persistence Test"
echo "========================="
echo ""

# Check if database exists
if [ -f "plon.db" ]; then
    echo "✓ Database file exists: plon.db"
    
    # Check the size
    SIZE=$(du -h plon.db | cut -f1)
    echo "  Database size: $SIZE"
    
    # Count tasks in database
    echo ""
    echo "Checking database contents..."
    sqlite3 plon.db "SELECT COUNT(*) FROM tasks;" 2>/dev/null && echo "  ✓ Tasks table accessible"
    
    TASK_COUNT=$(sqlite3 plon.db "SELECT COUNT(*) FROM tasks;" 2>/dev/null)
    echo "  Number of tasks in database: $TASK_COUNT"
    
    # Show task status distribution
    echo ""
    echo "Task distribution by status:"
    sqlite3 plon.db "SELECT status, COUNT(*) FROM tasks GROUP BY status;" 2>/dev/null | while IFS='|' read status count; do
        echo "  - $status: $count tasks"
    done
    
    # Show some sample tasks
    echo ""
    echo "Sample tasks (first 5):"
    sqlite3 plon.db "SELECT title, status FROM tasks LIMIT 5;" 2>/dev/null | while IFS='|' read title status; do
        echo "  - [$status] $title"
    done
    
else
    echo "✗ Database file not found"
    echo "  Run the application first to create the database"
fi

echo ""
echo "========================="
echo "Test Instructions:"
echo "1. Run: cargo run --bin plon-desktop"
echo "2. Click on 'Kanban' tab"
echo "3. Drag a task to a different column"
echo "4. Navigate to another tab (e.g., 'List')"
echo "5. Navigate back to 'Kanban'"
echo "6. Verify the task is still in the new column"
echo "7. Close the app and run it again"
echo "8. The task should still be in the new column"
echo ""
echo "Run this script again after making changes to verify persistence"