#!/bin/bash

# Build script for creating Plon.app for macOS
set -e

echo "🚀 Building Plon for macOS..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

# Create app bundle structure
APP_NAME="Plon"
APP_BUNDLE="$APP_NAME.app"
CONTENTS_DIR="$APP_BUNDLE/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

echo -e "${YELLOW}Creating app bundle structure...${NC}"
rm -rf "$APP_BUNDLE"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Build the release binary
echo -e "${YELLOW}Building release binary...${NC}"
cargo build --release

# Copy the binary
echo -e "${YELLOW}Copying binary to app bundle...${NC}"
cp "target/release/plon" "$MACOS_DIR/$APP_NAME"

# Copy database file if it exists
if [ -f "plon.db" ]; then
    echo -e "${YELLOW}Copying database...${NC}"
    cp "plon.db" "$RESOURCES_DIR/"
fi

# Copy migrations folder
if [ -d "migrations" ]; then
    echo -e "${YELLOW}Copying migrations...${NC}"
    cp -r "migrations" "$RESOURCES_DIR/"
fi

# Create Info.plist
echo -e "${YELLOW}Creating Info.plist...${NC}"
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>Plon</string>
    <key>CFBundleIdentifier</key>
    <string>com.plon.app</string>
    <key>CFBundleName</key>
    <string>Plon</string>
    <key>CFBundleDisplayName</key>
    <string>Plon</string>
    <key>CFBundleVersion</key>
    <string>1.0.0</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.productivity</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>NSMainNibFile</key>
    <string>MainMenu</string>
</dict>
</plist>
EOF

# Create a launcher script that sets up the environment
echo -e "${YELLOW}Creating launcher script...${NC}"
cat > "$MACOS_DIR/plon-launcher" << 'EOF'
#!/bin/bash

# Get the directory where the app bundle is located
APP_DIR="$(cd "$(dirname "$0")/../.." && pwd)"
RESOURCES_DIR="$APP_DIR/Contents/Resources"
BINARY_PATH="$APP_DIR/Contents/MacOS/Plon"

# Set up environment
export RUST_BACKTRACE=1

# Check if we need to use the bundled database
if [ -f "$RESOURCES_DIR/plon.db" ]; then
    export DATABASE_URL="sqlite:$RESOURCES_DIR/plon.db"
fi

# Change to Resources directory for migrations
cd "$RESOURCES_DIR"

# Launch the actual binary
exec "$BINARY_PATH"
EOF

chmod +x "$MACOS_DIR/plon-launcher"

# Update Info.plist to use the launcher
sed -i '' 's|<string>Plon</string>|<string>plon-launcher</string>|' "$CONTENTS_DIR/Info.plist"

echo -e "${GREEN}✅ App bundle created successfully!${NC}"
echo -e "${GREEN}📁 Location: $PROJECT_ROOT/$APP_BUNDLE${NC}"

# Ask if user wants to install to Applications
echo ""
echo -e "${YELLOW}Would you like to install Plon to /Applications? (y/n)${NC}"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    echo -e "${YELLOW}Installing to /Applications...${NC}"
    rm -rf "/Applications/$APP_BUNDLE"
    cp -r "$APP_BUNDLE" "/Applications/"
    echo -e "${GREEN}✅ Plon installed to /Applications successfully!${NC}"
    echo -e "${GREEN}🚀 You can now launch Plon from Launchpad or Spotlight!${NC}"
else
    echo -e "${YELLOW}App bundle created but not installed.${NC}"
    echo -e "${YELLOW}To install manually, run: cp -r '$APP_BUNDLE' /Applications/${NC}"
fi