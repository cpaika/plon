#!/bin/bash

# Script to create a simple app icon for Plon
set -e

echo "Creating Plon app icon..."

# Create a simple PNG icon using printf (base64 encoded blue square)
# This is a 16x16 blue PNG
BASE64_ICON="iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAABGdBTUEAALGPC/xhBQAAAAlwSFlzAAALEwAACxMBAJqcGAAAADJJREFUOE9j/P//PwMlgImSzqMGkBkAxPqP4v9QgI6B+jGcRa0AYnOo5kJqBRCzFQkAAKOZB3rnTkxjAAAAAElFTkSuQmCC"

# Decode the base64 to create a basic icon
echo "$BASE64_ICON" | base64 -d > AppIcon.png

# Try to use sips to resize it for a basic icon
if command -v sips &> /dev/null; then
    sips -z 512 512 AppIcon.png --out AppIcon_512.png 2>/dev/null || true
    
    # Try to convert to icns if iconutil is available
    if command -v iconutil &> /dev/null; then
        mkdir -p AppIcon.iconset
        sips -z 16 16 AppIcon.png --out AppIcon.iconset/icon_16x16.png 2>/dev/null || true
        sips -z 32 32 AppIcon.png --out AppIcon.iconset/icon_16x16@2x.png 2>/dev/null || true
        sips -z 32 32 AppIcon.png --out AppIcon.iconset/icon_32x32.png 2>/dev/null || true
        sips -z 64 64 AppIcon.png --out AppIcon.iconset/icon_32x32@2x.png 2>/dev/null || true
        sips -z 128 128 AppIcon.png --out AppIcon.iconset/icon_128x128.png 2>/dev/null || true
        sips -z 256 256 AppIcon.png --out AppIcon.iconset/icon_128x128@2x.png 2>/dev/null || true
        sips -z 256 256 AppIcon.png --out AppIcon.iconset/icon_256x256.png 2>/dev/null || true
        sips -z 512 512 AppIcon.png --out AppIcon.iconset/icon_256x256@2x.png 2>/dev/null || true
        sips -z 512 512 AppIcon.png --out AppIcon.iconset/icon_512x512.png 2>/dev/null || true
        sips -z 1024 1024 AppIcon.png --out AppIcon.iconset/icon_512x512@2x.png 2>/dev/null || true
        
        iconutil -c icns AppIcon.iconset -o AppIcon.icns 2>/dev/null || {
            echo "Warning: Could not create icns file"
        }
        rm -rf AppIcon.iconset
    fi
fi

# Clean up temporary files
rm -f AppIcon.png AppIcon_512.png

if [ -f "AppIcon.icns" ]; then
    echo "✅ Basic icon created: AppIcon.icns"
else
    echo "⚠️  Icon creation skipped (optional)"
fi