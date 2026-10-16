#!/bin/bash

# Run OpenCADStudio with logging enabled
# Logs will be captured to opencadstudio_logs.txt

echo "🚀 Starting OpenCADStudio with logging..."
echo "Logging to: opencadstudio_logs.txt"
echo ""
echo "When app opens:"
echo "  1. File → Open"
echo "  2. Select test_1k.dxf or test_10k.dxf"
echo "  3. Watch progress modal"
echo "  4. Observe load time and responsiveness"
echo ""
echo "Press Ctrl+C to stop logging"
echo "==========================================="
echo ""

# Set RUST_LOG for verbose output
export RUST_LOG=debug,opencadstudio=trace

# Run app and capture all output
/Users/voquochieu/Claude/target-game/OpenCADStudio/target/release/opencadstudio 2>&1 | tee opencadstudio_logs.txt

echo ""
echo "==========================================="
echo "✅ Logging complete. See opencadstudio_logs.txt for details"
