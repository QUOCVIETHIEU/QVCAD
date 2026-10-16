# Phase 2: File Loading Optimization - Implementation Guide

**Phase:** 2 (File Loading Optimization)  
**Duration:** 2 weeks  
**Goal:** 3-5x faster file loading with smooth UI and progress visualization  
**Status:** 🚀 Kickoff

---

## Overview

**Current Flow (Blocking):**
```
User clicks "Open File"
  ↓ (Main Thread - UI FROZEN)
DxfReader::from_file() loads entire file into memory
  ↓ (Main Thread - UI FROZEN)
Parser reads all entities (2.02ms+ for large files)
  ↓ (Main Thread - UI FROZEN)
Tessellation happens (4.17ms+ for curves)
  ↓ (Main Thread - UI FROZEN)
Scene updates
  ↓
UI re-renders (finally)
Result: 5-8 seconds of complete UI freeze ❌
```

**Target Flow (Non-Blocking):**
```
User clicks "Open File"
  ↓
Background thread: DxfReader loads file
  ↓ (Main thread - UI RESPONSIVE)
Background thread: Parse entities in batches
  ↓ (Main thread - continues) - Progress bar: "Loading entities 1000/10000"
Background thread: Tessellate progressively (coarse first)
  ↓ (Main thread - continues) - Progress bar: "Tessellating 1000/10000"
Stream entities to scene as ready
  ↓ (Main thread - continues) - Viewport shows entities as they arrive
UI updates incrementally
Result: Smooth progress, viewport interactive immediately ✅
```

---

## Architecture Design

### 1. Background File Loading Thread

**File:** `src/io/background_loader.rs` (NEW)

```rust
/// Manages background file loading with streaming entity delivery
pub struct BackgroundFileLoader {
    /// Channel to send parsed entities
    tx: mpsc::Sender<LoaderEvent>,
    /// Handle to background thread
    thread_handle: Option<JoinHandle<()>>,
}

pub enum LoaderEvent {
    /// Progress update: (entities_loaded, total_entities)
    Progress { loaded: usize, total: usize },
    /// Batch of entities ready to add to scene
    EntitiesBatch(Vec<(Handle, EntityType)>),
    /// Tessellation progress: (tessellated, total)
    TessellationProgress { done: usize, total: usize },
    /// File loading complete
    Complete { doc: CadDocument },
    /// Error occurred
    Error(String),
}

impl BackgroundFileLoader {
    /// Start loading a file in background
    pub fn start_load(path: PathBuf) -> (Self, mpsc::Receiver<LoaderEvent>) {
        let (tx, rx) = mpsc::channel(1000); // Buffered queue
        
        let thread_handle = std::thread::spawn(move || {
            // 1. Load file from disk (can be slow)
            // 2. Parse entities in batches, send progress
            // 3. Tessellate in parallel, send batches
            // 4. Send completion
        });
        
        (Self { tx, thread_handle: Some(thread_handle) }, rx)
    }
    
    /// Cancel loading (if user closes before complete)
    pub fn cancel(&mut self) {
        self.thread_handle.take();
    }
}
```

### 2. Streaming Entity Addition

**File:** `src/scene/streaming.rs` (NEW)

```rust
/// Accumulates entities in batches before adding to scene
pub struct StreamingEntityBatcher {
    /// Accumulated entities waiting to be added
    batch: Vec<(Handle, EntityType)>,
    /// Batch size (add to scene every N entities)
    batch_size: usize,
    /// Called when batch is ready
    on_batch: Option<Box<dyn Fn(Vec<(Handle, EntityType)>) + Send>>,
}

impl StreamingEntityBatcher {
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch: Vec::with_capacity(batch_size),
            batch_size,
            on_batch: None,
        }
    }
    
    /// Add an entity, returns true if batch ready
    pub fn add_entity(&mut self, handle: Handle, entity: EntityType) -> bool {
        self.batch.push((handle, entity));
        
        if self.batch.len() >= self.batch_size {
            // Trigger batch addition
            true
        } else {
            false
        }
    }
    
    /// Flush remaining entities
    pub fn flush(&mut self) -> Vec<(Handle, EntityType)> {
        std::mem::take(&mut self.batch)
    }
}
```

### 3. Progressive Tessellation

**File:** `src/scene/progressive_tessellation.rs` (NEW)

```rust
/// Manages progressive tessellation:
/// 1. Coarse version rendered immediately (low-poly)
/// 2. Fine version tessellated in background
/// 3. Swap when ready
pub struct ProgressiveTessellator {
    /// Coarse tessellations (ready now)
    coarse: HashMap<Handle, WireSet>,
    /// Fine tessellations (being computed)
    in_progress: HashMap<Handle, ProgressStatus>,
    /// Thread pool for tessellation
    pool: rayon::ThreadPool,
}

pub enum ProgressStatus {
    Queued,
    InProgress { percent: f32 },
    Ready(WireSet),
}

impl ProgressiveTessellator {
    pub fn new(num_threads: usize) -> Self {
        Self {
            coarse: HashMap::new(),
            in_progress: HashMap::new(),
            pool: rayon::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
        }
    }
    
    /// Queue entity for coarse tessellation (immediate)
    pub fn tessellate_coarse(&mut self, handle: Handle, entity: &EntityType) {
        // Simplified tessellation: fewer segments
        let coarse_wires = tessellate_coarse(entity);
        self.coarse.insert(handle, coarse_wires);
    }
    
    /// Queue entity for fine tessellation (background)
    pub fn tessellate_fine(&mut self, handle: Handle, entity: EntityType) {
        self.in_progress.insert(handle, ProgressStatus::Queued);
        
        self.pool.spawn(move || {
            // Full tessellation
            let fine_wires = tessellate_fine(&entity);
            // Send to channel (implement callback)
        });
    }
    
    /// Check for completed tessellations
    pub fn poll_ready(&mut self) -> HashMap<Handle, WireSet> {
        let mut ready = HashMap::new();
        
        for (handle, status) in self.in_progress.iter_mut() {
            if let ProgressStatus::Ready(wires) = status {
                ready.insert(*handle, wires.clone());
            }
        }
        
        for handle in ready.keys() {
            self.in_progress.remove(handle);
        }
        
        ready
    }
}
```

### 4. UI Progress Visualization

**File:** `src/ui/loading_progress.rs` (NEW)

```rust
#[derive(Debug, Clone)]
pub struct LoadingProgress {
    pub phase: LoadingPhase,
    pub current: usize,
    pub total: usize,
    pub elapsed_ms: u128,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LoadingPhase {
    /// Reading file from disk
    ReadingFile,
    /// Parsing DXF/DWG entities
    ParsingEntities,
    /// Tessellating geometry
    Tessellating,
    /// Completed
    Done,
}

impl LoadingProgress {
    pub fn percent(&self) -> f32 {
        if self.total == 0 {
            return 0.0;
        }
        (self.current as f32 / self.total as f32) * 100.0
    }
    
    pub fn status_text(&self) -> String {
        match self.phase {
            LoadingPhase::ReadingFile => {
                format!("Reading file... ({:.1}% complete)", self.percent())
            }
            LoadingPhase::ParsingEntities => {
                format!("Parsing entities... ({}/{}, {:.1}%)", 
                    self.current, self.total, self.percent())
            }
            LoadingPhase::Tessellating => {
                format!("Rendering geometry... ({}/{}, {:.1}%)", 
                    self.current, self.total, self.percent())
            }
            LoadingPhase::Done => "Complete".to_string(),
        }
    }
    
    pub fn eta_seconds(&self) -> Option<f32> {
        if self.current == 0 || self.elapsed_ms == 0 {
            return None;
        }
        
        let rate_per_ms = self.current as f32 / self.elapsed_ms as f32;
        let remaining = (self.total - self.current) as f32;
        let eta_ms = remaining / rate_per_ms;
        
        Some(eta_ms / 1000.0)
    }
}
```

---

## Implementation Steps

### Step 1: Create Background Loader (1-2 days)

**Files to modify/create:**
- [ ] Create `src/io/background_loader.rs`
- [ ] Modify `src/io/mod.rs` to expose async loader
- [ ] Add `mpsc` channel for events
- [ ] Implement batch parsing

**Key code locations:**
- Reference: `src/io/mod.rs:1390` (read_dxf_path)
- Goal: Make it send `LoaderEvent::EntitiesBatch` every 1000 entities

**Acceptance criteria:**
- Background thread loads file without blocking UI
- Progress events sent via channel every N entities
- Can load test_10k.dxf without freezing

### Step 2: Implement Streaming Addition (1-2 days)

**Files to create:**
- [ ] Create `src/scene/streaming.rs`
- [ ] Implement `StreamingEntityBatcher`
- [ ] Modify scene::add_entity to batch

**Integration points:**
- Modify `src/scene/mod.rs` to accept batched entities
- Use existing `bump_geometry_no_blocks()` instead of per-entity bumps
- Reduce epoch updates from 10K to 10 for 10K entities

**Acceptance criteria:**
- Entities added in batches of 1000
- Scene only re-tessellates 10 times instead of 10K times
- Estimated 100x speedup for entity addition

### Step 3: Progressive Tessellation (2-3 days)

**Files to create:**
- [ ] Create `src/scene/progressive_tessellation.rs`
- [ ] Implement coarse tessellation (low-poly, fast)
- [ ] Implement fine tessellation (full detail, slower)

**Algorithm - Coarse Tessellation:**
```
For curves: Use fewer segments (reduce by 4x)
For circles: 8 segments instead of 32
For arcs: 4 segments instead of 16
For lines: No tessellation (keep as line)
For polylines: Use vertices as-is (skip inner subdivision)
```

**Algorithm - Fine Tessellation:**
```
Full precision tessellation with default parameters
Run in thread pool (rayon)
Swap with coarse version when ready
```

**Acceptance criteria:**
- Coarse wireframe displays in <100ms for 10K entities
- Fine tessellation happens in background
- Users see geometry immediately (coarse), gets better quality later (fine)

### Step 4: UI Progress Display (1-2 days)

**Files to create:**
- [ ] Create `src/ui/loading_progress.rs`
- [ ] Modify `src/app/mod.rs` to show progress modal

**Progress modal shows:**
- Loading bar (current / total)
- Phase name (Reading / Parsing / Tessellating)
- Percentage complete
- Estimated time remaining
- "Cancel" button to abort load

**Acceptance criteria:**
- Modal appears immediately on file open
- Updates every 500ms without stuttering
- Shows realistic ETA

### Step 5: Integration & Testing (1-2 days)

**Integration points:**
1. Message handler `Message::OpenFile` → starts `BackgroundFileLoader`
2. Receive `LoaderEvent` → update UI + add to scene
3. Scene updates → re-render viewport
4. File load complete → dismiss progress modal

**Testing checklist:**
- [ ] Load test_1k.dxf: should complete in <300ms
- [ ] Load test_5k.dxf: should complete in <800ms
- [ ] Load test_10k.dxf: should complete in <2s (vs 5-8s currently)
- [ ] Load test_25k.dxf: should complete in <5s
- [ ] UI remains responsive throughout
- [ ] Viewport shows entities as they load
- [ ] Cancel button works
- [ ] Memory usage stays <400MB peak

---

## Code Modifications Required

### 1. Modify `src/app/mod.rs`

Add to `Message` enum:
```rust
/// File loading events from background thread
FileLoadingProgress(LoadingProgress),
FileLoadingEvent(LoaderEvent),
FileLoadingError(String),
```

In `update()` function:
```rust
Message::OpenFile => {
    let (loader, rx) = BackgroundFileLoader::start_load(path);
    self.current_loader = Some(loader);
    self.loader_rx = Some(rx);
    self.loading_progress = Some(LoadingProgress::new());
    self.show_loading_modal = true;
    // Spawn task to poll rx channel
    Task::perform(poll_loader_events(rx), |msg| msg)
}
```

### 2. Modify `src/scene/mod.rs`

Add streaming support:
```rust
pub fn add_entities_batch(&mut self, batch: Vec<(Handle, EntityType)>) {
    for (handle, entity) in batch {
        self.add_entity(entity, handle);
    }
    // Single bump_geometry_no_blocks() instead of per-entity
    self.bump_geometry_no_blocks();
}
```

### 3. Modify `Cargo.toml`

Add channel for event passing:
```toml
crossbeam-channel = "0.5"  # Better for this use case than std::sync::mpsc
rayon = "1.12"  # Already in project
```

---

## Performance Targets

| Metric | Current | Target | Success |
|--------|---------|--------|---------|
| Test 1K load | 1s | <300ms | 3x faster |
| Test 5K load | 3-4s | <800ms | 4-5x faster |
| Test 10K load | 5-8s | <2s | 3-4x faster |
| Test 25K load | 15-20s | <5s | 3-4x faster |
| Viewport first render | 2-3s | <500ms | 4-6x faster |
| Peak memory | 600-800MB | <400MB | 50% reduction |
| UI responsiveness | Frozen | Smooth | 100% improvement |

---

## Rollout Plan

### Week 1: Implement Core
- Background loader (Step 1)
- Streaming addition (Step 2)
- Basic testing

**Benchmark:** Expect 2-3x improvement

### Week 2: Polish & Complete
- Progressive tessellation (Step 3)
- UI progress display (Step 4)
- Integration & testing (Step 5)

**Benchmark:** Expect 3-5x improvement + smooth UX

---

## Success Criteria

✅ File loading doesn't freeze UI  
✅ Progress modal appears with realistic ETA  
✅ Viewport shows entities incrementally  
✅ test_10k.dxf loads in <2 seconds  
✅ Memory peak <400MB  
✅ All tests pass  
✅ Smooth 60fps during load progress display  

---

## Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| Threading complexity | Use crossbeam channels, add unit tests |
| Memory spikes | Profile with heaptrack, stream in smaller batches |
| Tesselation bugs | Start with coarse tessellation (simpler code) |
| Progress bar inaccuracy | Calibrate with test files first |
| Regression in performance | Keep Phase 1 benchmarks running |

---

## Next Action

**Step 1 begins now:** Create `src/io/background_loader.rs` with background file parsing support.

Expected completion: Phase 2 in 2 weeks → **3-5x faster file loading** + smooth UI
