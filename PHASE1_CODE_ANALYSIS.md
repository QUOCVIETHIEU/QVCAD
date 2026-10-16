# Phase 1: Code Structure Analysis

## 1. Geometry Update Pipeline

### Current Architecture
```
Document changes
  ↓
bump_geometry() or bump_geometry_no_blocks()
  ↓
Epoch increment (geometry_epoch)
  ↓
Cache invalidation (tess_memo, resident_tess_memo)
  ↓
On next render:
  - Full tessellation of changed entities (if bump_geometry)
  - Visible-only tessellation (if bump_geometry_no_blocks)
  - Quadtree stays the same OR full rebuild if needed
```

### Key Code Locations
- **Epoch system:** `src/scene/mod.rs:700-800` - Global epoch tracker
- **bump_geometry():** `src/scene/mod.rs:2700+` - Triggers full rebuild
- **Tessellation cache:** `src/scene/mod.rs:2300+` - Memoization structure
- **Quadtree update issue:** `src/scene/pick/quadtree.rs:84-88` - EXPLICITLY notes full rebuilds

### Problem: Full Rebuild on Every Entity Addition
Line 84-88 in `quadtree.rs`:
```rust
// "production callers haven't landed yet (Scene::add/erase/transform paths
// still trigger an epoch-based full rebuild rather than mutating the index
// in place). The `#[allow(dead_code)]` is intentional"
```

**Impact:** When loading a 10K entity file:
1. Entity 1 added → Full quadtree rebuild
2. Entity 2 added → Full quadtree rebuild
3. ...
4. Entity 10K added → Full quadtree rebuild

**Time Complexity:** O(n²) where n = number of entities being loaded

**Estimated Cost:** 
- Simple rebuild: O(n log n)
- 10 entities: 33 rebuilds = 330 comparisons
- 10K entities: 50M rebuilds = 500M comparisons

---

## 2. Tessellation Pipeline

### Current Flow
```
Entity to render
  ↓
Check tess_memo cache
  ↓
If miss → tessellate_entity()
  ↓
For curves: lyon_tessellation (main thread, blocks UI)
  ↓
GPU upload
  ↓
Render
```

### Key Code Locations
- **tessellate_entity:** Search in `src/scene/mod.rs` (line ~4500+)
- **Lyon tessellation:** `src/scene/preview.rs:296+`
- **Caching:** `src/scene/mod.rs:2250+` - tess_memo HashMap

### Problem: Synchronous Tessellation Blocks UI
- **Curves (splines, arcs, polylines):** Require CPU tessellation
- **Thread:** Runs on main thread (no parallelization visible)
- **No LOD:** Generates full detail immediately
- **Memory:** Stores entire tessellation upfront

**Impact on large files:**
- Each curved entity = 100-1000ms of CPU work
- Loading 1000 arcs = 100-1000 seconds of blocking
- UI freezes completely

---

## 3. Spatial Index (Quadtree)

### Structure
```rust
pub struct QuadTree {
    nodes: Vec<Node>,              // Flat vector for cache efficiency
    locator: FxHashMap<Handle, (u32, u32)>,  // O(1) lookup
    overflow: Vec<Item>,           // Items outside bounds
}
```

### Characteristics
- **Leaf capacity:** 32 items (good default)
- **Max depth:** 16 levels (prevents pathological cases)
- **Straddling items:** Kept at parent level if span multiple quadrants
- **Rebalancing:** NO - once split, doesn't merge back

### Not Used For
- File loading (full rebuild instead)
- Incremental entity updates
- Viewport culling (frustum done elsewhere)

### Optimization Opportunity
**Current:** Full rebuild on each entity add
**Potential:** Incremental insert/update/remove (already implemented but unused!)

```rust
pub fn insert(&mut self, handle: Handle, aabb: Aabb) { ... }  // Exists but unused!
pub fn update(&mut self, handle: Handle, aabb: Aabb) { ... }  // Exists but unused!
pub fn remove(&mut self, handle) { ... }                       // Exists but unused!
```

---

## 4. LOD & Culling System

### What Exists
1. **Frustum culling:** `src/io/xref.rs` mentions "frustum + LOD culling keep GPU cost bounded"
2. **LOD pixel-size:** `src/scene/mod.rs:2578-2579` - viewport size drives LOD selection
3. **Interaction LOD:** Hatch suppression during pan/zoom (`OCS_HATCH_LOD` env var)
4. **Block LOD:** Coarse cached solid LOD for hover vs click selection

### Current Thresholds
- Default pixel-size culling: TBD (needs investigation)
- Hatch LOD disabled by default (`OCS_HATCH_LOD` env var)
- Interaction LOD: Reduces complexity during camera movement

### Not Implemented
- Progressive/streaming tessellation during load
- Viewport-based tessellation order (prioritize visible)
- Dynamic LOD level selection based on entity complexity
- Adaptive threshold tuning

---

## 5. File Loading Pipeline

### Current Flow (Estimated)
```
Open DWG/DXF file
  ↓
Parse entities (codec layer)
  ↓
For each entity:
  - Add to document
  - bump_geometry() or mark_entity_dirty()
  ↓
On visibility:
  - Tessellate all
  - Build GPU buffers
  - Render
```

### Bottlenecks
1. **Serial entity addition:** No batching observed
2. **Per-entity epoch bump:** Causes quadtree rebuilds
3. **Upfront tessellation:** All entities tessellated before first render
4. **No streaming:** Viewport doesn't render until all data ready

### Potential Improvements
- Batch entity addition (single epoch bump)
- Defer tessellation until viewport display
- Progressive rendering (display what's ready)
- Background tessellation for off-screen entities

---

## 6. GPU Rendering & Buffers

### Current Stack
- **Graphics API:** wgpu (abstraction over Vulkan/DX12/Metal/GL)
- **Viewport rendering:** iced framework + custom GPU backend
- **Shader language:** WGSL
- **Buffer management:** StagingBelt + vertex/index buffers

### Observations
- Uses iced for UI (retained-mode framework)
- GPU backend selection auto-probes (safe mode handling)
- Renderer supports packed vertex format for lower-spec GPUs
- No visible instancing framework (repeated blocks = repeated geometry)

### Opportunities
- GPU instancing for block references
- Deferred rendering for complex materials
- Compute shader for hatch rasterization
- Persistent mapped buffers (if GPU supports)

---

## 7. File Size & Complexity

### Project Statistics
- **Total Rust files:** 730
- **Largest module:** `src/scene/mod.rs` (644 KB)
- **GPU backend:** `src/gpu_backend.rs` (58 KB)
- **Performance benches:** 17+ comprehensive benchmarks
- **Quadtree:** 15 KB (small, well-contained)

### Code Quality Indicators
✅ Spatial indexing already present  
✅ Epoch-based invalidation (good architecture)  
✅ Tessellation memoization  
✅ Comprehensive benchmarking  
⚠️ Quadtree not used for incremental updates  
⚠️ No parallel tessellation visible  
⚠️ Full rebuilds on entity add  
⚠️ Hatch LOD disabled by default  

---

## 8. Optimization Priority by Effort

### Quick Wins (30 min - 1 hour)
1. Enable hatch LOD by default (`OCS_HATCH_LOD=1`)
2. Increase pixel-size LOD threshold
3. Batch entity additions (single epoch bump)
4. Verify frustum culling is aggressive enough

### Medium Effort (1-2 days)
1. ✅ Use quadtree incremental updates
2. ✅ Parallelize tessellation with rayon
3. ✅ Implement progressive tessellation
4. ✅ Add viewport-aware tessellation priority

### High Effort (3-7 days)
1. ✅ GPU-based hatch rasterization
2. ✅ Hierarchical spatial index (BVH)
3. ✅ Streaming file load
4. ✅ GPU instancing for blocks

---

## 9. Benchmarking Opportunities

The project already has excellent benchmarks in `benches/performance_benchmarks.rs`:
1. Scene entity ingestion (10K entities)
2. Tessellation (14K + 7.5K entities)
3. Arc tessellation
4. Spatial index construction
5. Hit testing
6. Object snapping
7. Constraint solving
8. DXF I/O
9. Selection state
10. UI rendering
11. Viewport grid
12. Plot style rebuild
13. Model extents
14. Entity transformation
15. Draworder depth
16. Undo transactions
17. Viewport render pipeline

### Recommended Additions
- File load time (10K entity DXF from disk)
- Quadtree rebuild vs incremental update
- Tessellation cache hit/miss ratio
- GPU buffer upload time
- Viewport culling efficiency

---

## Next Steps for Phase 1

1. ✅ Compile & run full benchmarks (in progress)
2. ⏳ Identify top 3 slowest operations
3. ⏳ Create test DWG with 5K, 10K, 50K entities
4. ⏳ Profile file loading with flamegraph
5. ⏳ Generate detailed Phase 1 report
