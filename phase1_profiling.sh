#!/bin/bash

# Phase 1: OpenCADStudio Performance Analysis & Profiling
# This script profiles file loading, tessellation, and rendering performance

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROFILE_DIR="$PROJECT_DIR/profile_results"
FLAMEGRAPH_DIR="$PROFILE_DIR/flamegraphs"
BENCHMARK_DIR="$PROFILE_DIR/benchmarks"

echo "=========================================="
echo "Phase 1: OpenCADStudio Profiling"
echo "=========================================="
echo "Date: $(date)"
echo "Working Directory: $PROJECT_DIR"

# Create directories
mkdir -p "$PROFILE_DIR" "$FLAMEGRAPH_DIR" "$BENCHMARK_DIR"

# Step 1: Build release binary
echo ""
echo "Step 1: Building release binary..."
cargo build --release --bin OpenCADStudio 2>&1 | tail -5

# Step 2: Run existing benchmarks
echo ""
echo "Step 2: Running performance benchmarks (quick mode)..."
cargo bench --bench performance_benchmarks -- --quick 2>&1 | tee "$BENCHMARK_DIR/quick_benchmark.txt"

# Step 3: Run longer benchmarks
echo ""
echo "Step 3: Running full performance benchmarks..."
echo "(This may take 5-10 minutes...)"
cargo bench --bench performance_benchmarks 2>&1 | tee "$BENCHMARK_DIR/full_benchmark.txt" &
BENCH_PID=$!

# While benchmarks run, analyze the code structure
echo ""
echo "Step 4: Analyzing codebase for optimization opportunities..."

echo "=== File Sizes by Module ===" >> "$PROFILE_DIR/code_analysis.txt"
find src -name "*.rs" -exec wc -l {} + | sort -rn | head -20 >> "$PROFILE_DIR/code_analysis.txt"

echo "" >> "$PROFILE_DIR/code_analysis.txt"
echo "=== GPU Backend Information ===" >> "$PROFILE_DIR/code_analysis.txt"
grep -n "wgpu\|gpu\|tessellation\|spatial" src/*.rs src/**/*.rs 2>/dev/null | head -30 >> "$PROFILE_DIR/code_analysis.txt" || true

echo "" >> "$PROFILE_DIR/code_analysis.txt"
echo "=== Quadtree Usage ===" >> "$PROFILE_DIR/code_analysis.txt"
grep -rn "QuadTree\|quadtree\|spatial.*index" src --include="*.rs" | head -20 >> "$PROFILE_DIR/code_analysis.txt"

echo ""
echo "Step 5: Analyzing critical paths in the code..."
echo "=== Critical Bottlenecks ===" > "$PROFILE_DIR/bottleneck_analysis.txt"

echo "1. Scene entity ingestion:" >> "$PROFILE_DIR/bottleneck_analysis.txt"
grep -A5 "epoch.*rebuild\|full rebuild" src/scene/pick/quadtree.rs >> "$PROFILE_DIR/bottleneck_analysis.txt" || echo "Not found" >> "$PROFILE_DIR/bottleneck_analysis.txt"

echo "" >> "$PROFILE_DIR/bottleneck_analysis.txt"
echo "2. Tessellation:" >> "$PROFILE_DIR/bottleneck_analysis.txt"
grep -rn "lyon_tessellation\|tessellate" src --include="*.rs" | head -10 >> "$PROFILE_DIR/bottleneck_analysis.txt" || true

echo "" >> "$PROFILE_DIR/bottleneck_analysis.txt"
echo "3. LOD culling:" >> "$PROFILE_DIR/bottleneck_analysis.txt"
grep -rn "viewport.*cull\|LOD\|pixel.*size" src --include="*.rs" | head -10 >> "$PROFILE_DIR/bottleneck_analysis.txt" || true

# Wait for benchmarks to complete
echo ""
echo "Waiting for benchmarks to complete..."
wait $BENCH_PID

# Step 6: Generate flamegraph (if perf tools available)
echo ""
echo "Step 6: Attempting flamegraph generation..."
if command -v flamegraph &> /dev/null; then
    echo "Creating a simple test to flamegraph (reading a DXF file)..."
    # We'll create this later when we have test files
    echo "Flamegraph tools available - will use for file loading profiling"
else
    echo "Flamegraph not available - will use Rust benchmarks only"
fi

# Step 7: Summary
echo ""
echo "=========================================="
echo "Phase 1 Analysis Complete"
echo "=========================================="
echo ""
echo "Results saved to: $PROFILE_DIR"
echo ""
echo "Key files:"
echo "  - $BENCHMARK_DIR/full_benchmark.txt     (Performance benchmarks)"
echo "  - $BENCHMARK_DIR/quick_benchmark.txt    (Quick benchmark)"
echo "  - $PROFILE_DIR/code_analysis.txt        (Codebase structure)"
echo "  - $PROFILE_DIR/bottleneck_analysis.txt  (Critical paths)"
echo ""
echo "Next steps:"
echo "  1. Review benchmark results"
echo "  2. Identify top 3 slowest operations"
echo "  3. Create test DWG files with varying entity counts"
echo "  4. Profile specific bottlenecks with flamegraph"
echo ""
