#!/bin/bash

# Build and install script for Plon macOS app
set -e

echo "🚀 Building and installing Plon for macOS..."

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
echo -e "${YELLOW}Building release binary (this may take a few minutes)...${NC}"
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

# Copy icon if it exists
if [ -f "macos/AppIcon.icns" ]; then
    echo -e "${YELLOW}Copying app icon...${NC}"
    cp "macos/AppIcon.icns" "$RESOURCES_DIR/AppIcon.icns"
fi

# Create Info.plist
echo -e "${YELLOW}Creating Info.plist...${NC}"
cat > "$CONTENTS_DIR/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>plon-launcher</string>
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

# Use the user's home directory for the database
USER_DATA_DIR="$HOME/.plon"
mkdir -p "$USER_DATA_DIR"

# Copy migrations if they don't exist
if [ ! -d "$USER_DATA_DIR/migrations" ] && [ -d "$RESOURCES_DIR/migrations" ]; then
    cp -r "$RESOURCES_DIR/migrations" "$USER_DATA_DIR/"
fi

# Set database path to user's home directory
export DATABASE_URL="sqlite:$USER_DATA_DIR/plon.db"

# Change to user data directory
cd "$USER_DATA_DIR"

# Launch the actual binary
exec "$BINARY_PATH"
EOF

chmod +x "$MACOS_DIR/plon-launcher"

echo -e "${GREEN}✅ App bundle created successfully!${NC}"

# Install to Applications
echo -e "${YELLOW}Installing to /Applications...${NC}"
rm -rf "/Applications/$APP_BUNDLE"
cp -r "$APP_BUNDLE" "/Applications/"

# Clean up local app bundle
rm -rf "$APP_BUNDLE"

echo -e "${GREEN}✅ Plon has been installed to /Applications!${NC}"
echo -e "${GREEN}🎯 You can now launch Plon from:${NC}"
echo -e "${GREEN}   • Launchpad${NC}"
echo -e "${GREEN}   • Spotlight (press ⌘+Space and type 'Plon')${NC}"
echo -e "${GREEN}   • Applications folder${NC}"
echo ""
echo -e "${YELLOW}Note: The app data will be stored in ~/.plon/${NC}"