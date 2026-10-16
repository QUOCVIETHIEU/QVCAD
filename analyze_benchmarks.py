#!/usr/bin/env python3

"""
Analyze OpenCADStudio benchmark results
Extracts timing, identifies bottlenecks, ranks by performance impact
"""

import re
import json
from pathlib import Path
from collections import defaultdict

def parse_benchmark_output(text):
    """Extract benchmark results from cargo bench output"""

    results = {}

    # Pattern: name: "X.XXms" or "XXXus" or "XXXns"
    patterns = [
        (r'(\w+).*?(\d+\.?\d*)\s*ms', 1000),  # milliseconds
        (r'(\w+).*?(\d+\.?\d*)\s*µs', 0.001),  # microseconds
        (r'(\w+).*?(\d+\.?\d*)\s*us', 0.001),  # microseconds
        (r'(\w+).*?time.*?(\d+\.?\d*)\s*([mn]s)', None),  # generic time
    ]

    for line in text.split('\n'):
        if 'time' in line.lower() or 'bench' in line.lower():
            for pattern, multiplier in patterns:
                match = re.search(pattern, line)
                if match:
                    name = match.group(1)
                    value = float(match.group(2))
                    if multiplier:
                        value *= multiplier
                    results[name] = value
                    break

    return results

def identify_bottlenecks(results):
    """Rank benchmarks by time cost"""

    bottlenecks = sorted(
        results.items(),
        key=lambda x: x[1],
        reverse=True
    )

    return bottlenecks

def main():
    # Try to read benchmark file
    bench_file = Path('/tmp/phase1_benchmark_baseline.txt')

    if not bench_file.exists():
        print("❌ Benchmark file not found:", bench_file)
        print("   Run: cargo bench --bench performance_benchmarks -- --quick")
        return

    text = bench_file.read_text()

    # Parse results
    results = parse_benchmark_output(text)

    if not results:
        print("⏳ Benchmarks still running...")
        print("   Check progress: tail -f /tmp/phase1_benchmark_baseline.txt")
        return

    # Analyze
    bottlenecks = identify_bottlenecks(results)

    print("\n" + "="*60)
    print("OpenCADStudio Phase 1 Benchmark Analysis")
    print("="*60 + "\n")

    print("Top Bottlenecks (by time):\n")

    total_time = sum(v for _, v in bottlenecks)

    for i, (name, time_ms) in enumerate(bottlenecks[:10], 1):
        percentage = (time_ms / total_time * 100) if total_time > 0 else 0
        bar = "█" * int(percentage / 5)
        print(f"{i:2}. {name:40} {time_ms:10.2f}ms  {bar} {percentage:5.1f}%")

    print("\n" + "="*60)
    print(f"Total measured time: {total_time:.2f}ms")
    print("="*60)

    # Recommendations
    print("\n📊 Recommendations:\n")

    if bottlenecks[0][0] in ['scene_entity_ingestion', 'ingestion']:
        print("1. 🎯 PRIMARY: Entity ingestion bottleneck")
        print("   → Check incremental quadtree updates")
        print("   → Profile entity addition in bulk load")

    if any('tessellat' in name.lower() for name, _ in bottlenecks[:5]):
        print("2. 🎯 PRIMARY: Tessellation bottleneck")
        print("   → Parallelize with rayon")
        print("   → Implement progressive tessellation")
        print("   → Profile lyon_tessellation calls")

    if any('render' in name.lower() for name, _ in bottlenecks[:5]):
        print("3. 🎯 RENDERING: GPU pipeline bottleneck")
        print("   → Check buffer upload costs")
        print("   → Verify frustum culling effectiveness")

    print("\n💡 Next Steps:")
    print("   1. Generate flamegraph: cargo flamegraph --bin OpenCADStudio")
    print("   2. Profile test file: OpenCADStudio test_10k.dxf")
    print("   3. Correlate with benchmark results")

if __name__ == '__main__':
    main()
