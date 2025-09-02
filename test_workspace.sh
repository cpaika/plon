#!/bin/bash

echo "Testing Plon Workspace Creation"
echo "================================"
echo ""

# Clean up existing test directories (for testing)
echo "Cleaning up any existing test directories..."
rm -rf ~/plon-projects ~/plon-backups ~/plon-templates ~/.plon

echo ""
echo "Starting Plon to create workspace directories..."
echo ""

# Run the app briefly to trigger directory creation
timeout 3 cargo run --bin plon-web 2>&1 | grep -E "Created|workspace|directory" || true

echo ""
echo "Checking created directories..."
echo ""

# Check if directories were created
DIRS_TO_CHECK=(
    "$HOME/plon-projects"
    "$HOME/plon-backups"
    "$HOME/plon-templates"
    "$HOME/.plon"
    "$HOME/.plon/cache"
    "$HOME/.plon/logs"
)

ALL_CREATED=true
for dir in "${DIRS_TO_CHECK[@]}"; do
    if [ -d "$dir" ]; then
        echo "✅ $dir exists"
    else
        echo "❌ $dir not found"
        ALL_CREATED=false
    fi
done

echo ""

# Check for README
if [ -f "$HOME/plon-projects/README.md" ]; then
    echo "✅ README.md created in plon-projects"
    echo ""
    echo "README.md contents:"
    echo "-------------------"
    head -10 "$HOME/plon-projects/README.md"
    echo "..."
else
    echo "❌ README.md not found in plon-projects"
fi

echo ""
echo "================================"
if [ "$ALL_CREATED" = true ]; then
    echo "✅ All workspace directories created successfully!"
else
    echo "⚠️ Some directories were not created"
fi