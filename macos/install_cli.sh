#!/bin/bash

# Install a CLI launcher for Plon

echo "Installing Plon CLI launcher..."

# Create the launcher script
cat > /tmp/plon << 'EOF'
#!/bin/bash
# Plon CLI launcher
open -a Plon "$@"
EOF

# Make it executable
chmod +x /tmp/plon

# Install to /usr/local/bin (or create it if it doesn't exist)
if [ ! -d /usr/local/bin ]; then
    echo "Creating /usr/local/bin..."
    sudo mkdir -p /usr/local/bin
fi

echo "Installing launcher to /usr/local/bin/plon (requires sudo)..."
sudo mv /tmp/plon /usr/local/bin/plon

echo "✅ Plon CLI launcher installed!"
echo "You can now launch Plon from terminal by typing: plon"