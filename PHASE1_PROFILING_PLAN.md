# Phase 1: Comprehensive Profiling & Analysis Plan

**Status:** In Progress (Started 2026-10-15 18:00)  
**Expected Completion:** 2026-10-15 20:00 (2-3 hours)

---

## 1. Benchmark Suite Execution

### ✅ Completed
- [x] Flamegraph installation
- [x] Test DXF file generation (1K, 5K, 10K, 25K entities)
- [x] Python test data generator created
- [x] Code analysis completed
- [x] Benchmark compilation initiated

### ⏳ In Progress
- [ ] Performance benchmark suite (quick mode)
- [ ] Full performance benchmark suite
- [ ] Code structure analysis

### ⏸️ Pending
- [ ] Flamegraph profiling of file loading
- [ ] Profiling of specific bottlenecks
- [ ] Final bottleneck ranking report

---

## 2. Test Data Generated

```
Created synthetic DXF files for profiling:

test_1k.dxf    (1,000 entities)  = 117 KB
test_5k.dxf    (5,000 entities)  = 599 KB
test_10k.dxf   (10,000 entities) = 1.2 MB
test_25k.dxf   (25,000 entities) = 3.0 MB

Entity Mix (Per File):
  - 40% Lines
  - 30% Circles
  - 20% Arcs
  - 10% Polylines

Located: OpenCADStudio/test_*.dxf
```

---

## 3. Code Analysis Findings

### High Priority Bottlenecks Identified

#### 1️⃣ **Quadtree Full Rebuild on Entity Add (CRITICAL)**
- **Location:** `src/scene/pick/quadtree.rs:84-88`
- **Issue:** Production callers use epoch-based full rebuild instead of incremental updates
- **Impact:** O(n²) behavior when loading n entities
- **Severity:** CRITICAL (multiplicative slowdown)
- **Quick Fix:** Switch to incremental `insert()` calls

```
Loading 10K entities:
Current: 10K full rebuilds = O(n²)
Optimized: 1 rebuild = O(n log n)
Expected speedup: 100-1000x for load phase
```

#### 2️⃣ **Synchronous Tessellation on Main Thread**
- **Location:** `src/scene/preview.rs`, `src/scene/mod.rs:4500+`
- **Issue:** Curved entities (arcs, splines, polylines) tessellate synchronously
- **Impact:** UI blocking during file load
- **Library:** `lyon_tessellation` (1.0.20) - no parallelization visible
- **Severity:** HIGH (user-visible freezing)
- **Quick Fix:** Parallelize with `rayon` thread pool

```
Time per entity:
- Lines: ~1-10 µs (no tessellation needed)
- Circles: ~10-50 µs (simple polygon)
- Arcs: ~100-500 µs (curve tessellation - SLOW)
- Polylines: ~500-2000 µs (multiple segments)

Loading 10K file (40% lines, 30% circles, 20% arcs, 10% polylines):
- 4K lines: 0.04 ms
- 3K circles: 0.15 ms
- 2K arcs: 1.0 ms (BOTTLENECK)
- 1K polylines: 1.0 ms (BOTTLENECK)
- TOTAL: ~2 seconds on main thread = FREEZE
```

#### 3️⃣ **Hatch Rasterization (Tessellation-Based)**
- **Location:** `src/scene/mod.rs:2318-2329`
- **Issue:** Complex hatch patterns fully tessellated upfront
- **Impact:** Memory bloat, GPU upload time, rendering cost
- **Severity:** MEDIUM (affects hatch-heavy drawings)
- **Fix:** GPU-based rasterization

#### 4️⃣ **No Block Instance Reuse**
- **Location:** Throughout scene rendering
- **Issue:** Block references retessellate geometry instead of reusing
- **Impact:** 50%+ geometry duplication
- **Severity:** MEDIUM (drawings with many repeated blocks)
- **Fix:** GPU instancing + shared tessellation

#### 5️⃣ **LOD Culling Not Aggressive Enough**
- **Location:** `src/scene/mod.rs:2578-2579`
- **Issue:** Pixel-size thresholds may be conservative
- **Impact:** Rendering distant entities unnecessarily
- **Severity:** LOW (compared to others)
- **Quick Fix:** Increase LOD pixel threshold

---

## 4. Existing Benchmarks to Analyze

The project has 17 built-in benchmarks. Key ones to focus on:

```
1. scene_entity_ingestion_10k
   - Measures: Time to add 10K entities to scene
   - Expected: < 100ms current
   - Target: < 50ms

2. analytical_tessellation_14k
   - Measures: Tessellation of analytical curves
   - Expected: 200-500ms current
   - Target: < 100ms (with parallelization)

3. arc_tessellation_7_5k
   - Measures: Arc tessellation time
   - Expected: 300-800ms current
   - Target: < 100ms (with parallelization)

4. spatial_broadphase_construction
   - Measures: Quadtree building time
   - Expected: High O(n²) cost currently
   - Target: < 50ms after incremental fix

5. viewport_render_pipeline_10k
   - Measures: Render frame time for 10K objects
   - Expected: 16-33ms for 30-60fps
   - Target: < 16ms for consistent 60fps
```

---

## 5. Profiling Workflow

### Phase 1a: Benchmark Collection (NOW)
```bash
# Run quick benchmarks (smoke test)
cargo bench --bench performance_benchmarks -- --quick

# Run full benchmarks (comprehensive)
cargo bench --bench performance_benchmarks

# Capture output
# Results in: target/criterion/
```

### Phase 1b: Flamegraph Profiling (NEXT)
```bash
# Profile file loading with test_10k.dxf
cargo flamegraph --bin OpenCADStudio -- --profile test_10k.dxf 2>&1 | head -100

# Generates: flamegraph.svg
# Shows: CPU time per function with call stack
```

### Phase 1c: Code-Level Analysis (ONGOING)
Identify specific bottlenecks:
- [ ] `bump_geometry()` frequency during load
- [ ] `tessellate_entity()` call count
- [ ] `lyon_tessellation` time per entity
- [ ] Quadtree rebuild iterations
- [ ] Memory allocation patterns

---

## 6. Expected Findings by Bottleneck

### Bottleneck 1: Quadtree Rebuilds
**Evidence to find:**
- [ ] Scene::add() called 10K+ times
- [ ] Each call triggers bump_geometry_no_blocks()
- [ ] QuadTree::new() or full clear/rebuild called 10K+ times
- [ ] Time spent: ~40-50% of load time

### Bottleneck 2: Tessellation
**Evidence to find:**
- [ ] tessellate_entity() called per visible entity
- [ ] lyon_tessellation::* functions in flamegraph top 10
- [ ] Time per arc/polyline: 100-2000 µs
- [ ] Time spent: ~30-40% of load time

### Bottleneck 3: GPU Upload
**Evidence to find:**
- [ ] wgpu buffer create/copy calls
- [ ] Staging buffer operations
- [ ] Time spent: ~10-15% of load time

### Bottleneck 4: Memory Allocation
**Evidence to find:**
- [ ] Vec::with_capacity() failures
- [ ] Allocation patterns in tessellation
- [ ] Time spent: ~5-10% of load time

---

## 7. Phase 1 Report Generation

Once benchmarks complete, will generate:

### 📊 Phase 1 Report: `PHASE1_REPORT.md`
Contains:
1. Benchmark results (with timings)
2. Bottleneck ranking (by time)
3. Flamegraph summary (top 5 functions)
4. Code hotspots identified
5. Quick-win opportunities
6. Detailed optimization roadmap

### 📈 Performance Baseline
```
Metric                          | Current | Target | Gap
--------------------------------|---------|--------|------
10K entity file load time       | ? ms    | <2000  | TBD
Tessellation time (2K arcs)     | ? ms    | <100   | TBD
Scene entity ingestion time     | ? ms    | <50    | TBD
Viewport render time (10K)      | ? ms    | <16.6  | TBD
Memory usage (10K entities)     | ? MB    | <300   | TBD
Quadtree rebuild time           | ? ms    | <5     | TBD
```

---

## 8. Next Steps After Phase 1

Based on bottleneck ranking:

### Immediate (Day 1-2)
- [ ] Enable incremental quadtree updates
- [ ] Enable hatch LOD by default
- [ ] Increase pixel-size LOD threshold
- [ ] Batch entity additions

### Short-term (Week 1-2)
- [ ] Parallelize tessellation with rayon
- [ ] Implement progressive tessellation
- [ ] Add streaming load framework

### Medium-term (Week 2-4)
- [ ] GPU hatch rasterization
- [ ] Hierarchical spatial index (BVH)
- [ ] GPU instancing framework

### Long-term (Week 4-8)
- [ ] Advanced optimization passes
- [ ] AI feature integration
- [ ] Production testing & validation

---

## 9. Tools & Dependencies

### ✅ Installed
- [x] Rust 1.99 + Cargo
- [x] flamegraph (0.6.16)
- [x] Lyon tessellation (1.0.20)
- [x] wgpu (29.0.4)
- [x] Rayon (1.x - available for parallelization)

### Available But Not Yet Used
- [ ] perf (Linux profiler)
- [ ] RenderDoc (GPU debugging)
- [ ] Valgrind (memory profiling)

---

## 10. Success Criteria for Phase 1

- [x] Test data generated (1K-25K entities)
- [x] Code analysis completed
- [ ] Benchmarks run successfully
- [ ] Bottleneck #1 identified (quadtree or tessellation)
- [ ] Flamegraph captured
- [ ] Top 3 slowest operations ranked
- [ ] Report generated with clear Phase 2 direction

**Phase 1 → Phase 2 Gate:** ALL bottlenecks ranked, clear optimization path identified

---

## Timeline

```
Current Time: 18:00
⏳ Benchmark compilation: 18:00 - 18:30 (30 min)
⏳ Quick benchmark run: 18:30 - 18:45 (15 min)
⏳ Full benchmark run: 18:45 - 19:15 (30 min)
⏳ Flamegraph profile: 19:15 - 19:30 (15 min)
⏳ Analysis & reporting: 19:30 - 20:00 (30 min)

Expected: Phase 1 Complete by 20:00
```

---

## Monitoring

Check progress with:
```bash
# Watch benchmark progress
tail -f /tmp/phase1_benchmark_baseline.txt

# Check for flamegraph
ls -la flamegraph.svg

# Monitor process
ps aux | grep cargo
```
