#!/bin/bash

echo "Testing Settings Fix for Repository Context Issue"
echo "================================================"
echo ""

# Run the test suite
echo "1. Running settings context tests..."
cargo test settings_context_test --lib --quiet
if [ $? -eq 0 ]; then
    echo "✅ Settings context tests passed"
else
    echo "❌ Settings context tests failed"
    exit 1
fi

echo ""
echo "2. Running app integration tests..."
cargo test test_app_provides_repository_context --lib --quiet
if [ $? -eq 0 ]; then
    echo "✅ App integration tests passed"
else
    echo "❌ App integration tests failed"
    exit 1
fi

echo ""
echo "3. Building web application..."
cargo build --bin plon-web --quiet
if [ $? -eq 0 ]; then
    echo "✅ Web application builds successfully"
else
    echo "❌ Web application build failed"
    exit 1
fi

echo ""
echo "4. Building desktop application..."
cargo build --bin plon-desktop --quiet
if [ $? -eq 0 ]; then
    echo "✅ Desktop application builds successfully"
else
    echo "❌ Desktop application build failed"
    exit 1
fi

echo ""
echo "================================================"
echo "✅ All tests passed! Settings page should now work."
echo ""
echo "The fix:"
echo "- Added Repository context provider in App component"
echo "- App now initializes database connection on startup"
echo "- All settings components can access Repository via context"
echo ""
echo "To run the app:"
echo "  cargo run --bin plon-web    # For web version"
echo "  cargo run --bin plon-desktop # For desktop version"