#!/bin/bash

echo "🧪 Running E2E Tests for Drag and Drop Functionality"
echo "=================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create screenshots directory
mkdir -p e2e-tests/screenshots

# Check if npm dependencies are installed
if [ ! -d "node_modules" ]; then
    echo -e "${YELLOW}Installing npm dependencies...${NC}"
    npm install
fi

# Build the web app
echo -e "${YELLOW}Building Dioxus web app...${NC}"
cargo build --bin plon-web --release

# Run specific test suites
echo -e "\n${GREEN}Running Kanban Board Tests...${NC}"
npx playwright test e2e-tests/kanban-drag-drop.spec.ts --reporter=list

echo -e "\n${GREEN}Running Map View Tests...${NC}"
npx playwright test e2e-tests/map-drag.spec.ts --reporter=list

# Run all tests with detailed reporter
echo -e "\n${GREEN}Running All Tests with HTML Report...${NC}"
npx playwright test --reporter=html

echo -e "\n${GREEN}Tests complete!${NC}"
echo "View the HTML report: npx playwright show-report"
echo "Screenshots saved in: e2e-tests/screenshots/"