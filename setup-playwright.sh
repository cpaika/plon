#!/bin/bash

echo "Setting up Playwright for E2E testing..."

# Install npm dependencies
echo "Installing npm dependencies..."
npm install

# Install Playwright browsers
echo "Installing Playwright browsers..."
npx playwright install

# Install system dependencies for Playwright
echo "Installing system dependencies..."
npx playwright install-deps

echo "Playwright setup complete!"
echo ""
echo "To run tests:"
echo "  npm test                 # Run all tests"
echo "  npm run test:chromium    # Run tests in Chromium only"
echo "  npm run test:headed      # Run tests with browser UI"
echo "  npm run test:debug       # Debug tests"
echo ""
echo "To run the Dioxus web app for testing:"
echo "  cargo run --bin plon-web"
echo ""
echo "To run the Dioxus desktop app:"
echo "  cargo run --bin plon-desktop"