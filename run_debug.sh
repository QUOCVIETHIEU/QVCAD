#!/bin/bash

echo "🚀 Starting OpenCADStudio with DEBUG logging..."
echo ""
echo "When app opens:"
echo "  1. File → Open"
echo "  2. Select test_1k.dxf"
echo "  3. Check terminal output for DEBUG messages"
echo ""
echo "==========================================="
echo ""

# Run app and show all output including debug eprintln!
/Users/voquochieu/Claude/target-game/OpenCADStudio/target/release/opencadstudio 2>&1 | tee opencadstudio_debug.log

echo ""
echo "==========================================="
echo "Debug log saved to opencadstudio_debug.log"
