# Plon macOS App Installation

## ✅ Installation Complete!

Plon has been successfully installed as a macOS application.

## 📍 Installation Details

- **Location**: `/Applications/Plon.app`
- **Data Directory**: `~/.plon/` (your home directory)
- **Database**: `~/.plon/plon.db`

## 🚀 How to Launch Plon

You can launch Plon in several ways:

1. **From Launchpad**: Click the Launchpad icon in your dock and search for "Plon"
2. **From Spotlight**: Press `⌘ + Space` and type "Plon"
3. **From Finder**: Open Applications folder and double-click Plon
4. **From Terminal**: Type `open -a Plon`

## 📁 App Structure

The app bundle includes:
- The compiled Plon binary
- Database migrations
- A launcher script that sets up the environment
- A basic app icon

## 🔧 Files Created

- `/Applications/Plon.app` - The main application bundle
- `~/.plon/` - User data directory (created on first launch)
- `~/.plon/plon.db` - SQLite database (created on first launch)
- `~/.plon/migrations/` - Database migrations

## 🔄 Updating the App

To update Plon to a newer version:
```bash
cd /Users/cpaika/projects/plon
./macos/install_app.sh
```

## 🗑️ Uninstalling

To uninstall Plon:
1. Drag Plon from Applications to Trash
2. Optionally, remove user data: `rm -rf ~/.plon`

## 🛠️ Troubleshooting

If the app doesn't launch:
1. Check that you have macOS 10.15 or later
2. Try launching from Terminal to see error messages: `open -a Plon`
3. Check the database permissions: `ls -la ~/.plon/`

## 📝 Notes

- The app will create its database in your home directory on first launch
- All your tasks and data are stored in `~/.plon/plon.db`
- The app icon is a simple blue square (you can customize it later)