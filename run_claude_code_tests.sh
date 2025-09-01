#!/bin/bash

# Claude Code Integration Test Runner
# This script runs all Claude Code related tests

set -e

echo "========================================="
echo "Claude Code Integration Test Suite"
echo "========================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test categories
UNIT_TESTS=0
INTEGRATION_TESTS=0
UI_TESTS=0
FAILED_TESTS=0

# Function to run a test category
run_test_category() {
    local category=$1
    local test_pattern=$2
    
    echo -e "${YELLOW}Running $category tests...${NC}"
    
    if cargo test $test_pattern --lib --bins -- --nocapture 2>&1 | tee test_output.tmp; then
        echo -e "${GREEN}✓ $category tests passed${NC}"
        return 0
    else
        echo -e "${RED}✗ $category tests failed${NC}"
        return 1
    fi
}

# Clean up any previous test artifacts
echo "Cleaning up previous test artifacts..."
rm -f test_output.tmp
rm -rf /tmp/claude_code_test_*

# Run unit tests for Claude Code domain models
echo
echo "1. Domain Model Tests"
echo "---------------------"
if run_test_category "Domain model" "claude_code::tests"; then
    ((UNIT_TESTS++))
else
    ((FAILED_TESTS++))
fi

# Run service layer tests
echo
echo "2. Service Layer Tests"
echo "----------------------"
if run_test_category "Service layer" "claude_code_service::tests"; then
    ((UNIT_TESTS++))
else
    ((FAILED_TESTS++))
fi

# Run repository tests
echo
echo "3. Repository Tests"
echo "-------------------"
if run_test_category "Repository" "claude_code_repository::tests"; then
    ((UNIT_TESTS++))
else
    ((FAILED_TESTS++))
fi

# Run integration tests
echo
echo "4. Integration Tests"
echo "--------------------"
if cargo test --test claude_code_integration_tests -- --nocapture 2>&1 | tee test_output.tmp; then
    echo -e "${GREEN}✓ Integration tests passed${NC}"
    ((INTEGRATION_TESTS++))
else
    echo -e "${RED}✗ Integration tests failed${NC}"
    ((FAILED_TESTS++))
fi

# Run UI tests
echo
echo "5. UI Tests"
echo "-----------"
if cargo test --test claude_code_ui_tests -- --nocapture 2>&1 | tee test_output.tmp; then
    echo -e "${GREEN}✓ UI tests passed${NC}"
    ((UI_TESTS++))
else
    echo -e "${RED}✗ UI tests failed${NC}"
    ((FAILED_TESTS++))
fi

# Test mock scripts
echo
echo "6. Mock Script Tests"
echo "--------------------"

# Test mock Claude CLI
echo "Testing mock Claude CLI..."
if ./tests/mocks/mock_claude_cli.sh code --file test.md --instructions inst.md --mode success > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Mock Claude CLI working${NC}"
else
    echo -e "${RED}✗ Mock Claude CLI failed${NC}"
    ((FAILED_TESTS++))
fi

# Test mock gh CLI
echo "Testing mock gh CLI..."
if ./tests/mocks/mock_gh_cli.sh pr create --title "Test" --body "Test" --base main --head feature > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Mock gh CLI working${NC}"
else
    echo -e "${RED}✗ Mock gh CLI failed${NC}"
    ((FAILED_TESTS++))
fi

# Run stress test (optional, disabled by default)
if [ "$1" == "--stress" ]; then
    echo
    echo "7. Stress Tests"
    echo "---------------"
    echo "Running stress test with 10 concurrent sessions..."
    
    cargo test --test claude_code_integration_tests test_multiple_concurrent_sessions -- --nocapture
fi

# Clean up test output file
rm -f test_output.tmp

# Summary
echo
echo "========================================="
echo "Test Summary"
echo "========================================="
echo -e "Unit Tests:        ${GREEN}$UNIT_TESTS passed${NC}"
echo -e "Integration Tests: ${GREEN}$INTEGRATION_TESTS passed${NC}"
echo -e "UI Tests:          ${GREEN}$UI_TESTS passed${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}All tests passed successfully!${NC}"
    exit 0
else
    echo -e "${RED}$FAILED_TESTS test categories failed${NC}"
    exit 1
fi