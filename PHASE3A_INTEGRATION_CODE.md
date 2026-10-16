# Phase 3A: Integration Code Examples

**Goal:** Wire Phase 2 components into the existing app message loop

---

## Step 1: Add Message Variants (app/mod.rs)

Find the `Message` enum around line 2383 and add these variants near the file-related messages:

```rust
pub enum Message {
    // ... existing messages ...
    
    // File opening
    OpenFile,
    OpenPathPicked(Option<(PathBuf, u64)>),
    OpenCancel,
    FileOpened(u64, Result<(String, PathBuf, CadDocument, crate::scene::DerivedCaches), crate::io::OpenLoadError>),
    
    // Phase 2: Background loading events
    #[cfg(not(target_arch = "wasm32"))]
    Phase2LoaderEvent(crate::io::background_loader::LoaderEvent),
    
    #[cfg(not(target_arch = "wasm32"))]
    Phase2ProgressTick,
    
    #[cfg(not(target_arch = "wasm32"))]
    Phase2CancelLoading,
    
    // ... rest of messages ...
}
```

**Location:** `src/app/mod.rs` around line 2515-2525 (after FileOpened)

---

## Step 2: Add App Fields (app/mod.rs)

Find the `pub struct App` definition and add these fields:

```rust
pub struct App {
    // ... existing fields ...
    
    // Phase 2: Background loading system
    #[cfg(not(target_arch = "wasm32"))]
    phase2_system: crate::app::phase2_loader_integration::Phase2FileLoadingSystem,
    
    // ... rest of fields ...
}
```

**Location:** Look for `pub struct App {` and add before the closing brace. Usually around line 300-400 in app/mod.rs

**In the impl App constructor:**

```rust
impl App {
    pub fn new(...) -> Self {
        Self {
            // ... existing fields ...
            
            #[cfg(not(target_arch = "wasm32"))]
            phase2_system: crate::app::phase2_loader_integration::Phase2FileLoadingSystem::new(),
            
            // ... rest ...
        }
    }
}
```

---

## Step 3: Wire OpenPathPicked Handler (app/update/mod.rs)

Replace the existing `Message::OpenPathPicked` handler (around line 1323):

```rust
Message::OpenPathPicked(Some((path, size_bytes))) => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use Phase 2 background loader
        self.phase2_system.start_load(path.clone());
        
        // Queue progress ticks to poll for events
        // This will be called frequently to check for loader events
        let task = Task::perform(
            async_progress_poller(),
            |_| Message::Phase2ProgressTick
        );
        
        return task;
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        // Web: use existing synchronous loading
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "unknown".into());
        let progress = std::sync::Arc::new(crate::io::OpenProgressState::new(
            super::OPEN_PHASE_READING,
        ));
        let open_id = self.next_open_id();
        self.opening = Some(super::OpenProgress {
            id: open_id,
            name: name.clone(),
            source_path: Some(path.clone()),
            size_bytes,
            state: progress.clone(),
            started: Instant::now(),
            recovery_error: None,
            recovery_read_stats: None,
            recovery_bytes: None,
            fingerprint: crate::io::edit_lock::FileFingerprint::capture(&path).ok(),
        });
        let size_label = format_size(size_bytes);
        self.command_line
            .push_info(crate::tf!("Opening \"{name}\" ({size_label})…").as_ref());
        let model_bg = self
            .default_bg_color
            .unwrap_or_else(|| self.model_space.resolve_model_bg(&self.active_theme));
        Task::perform(
            crate::io::open_path_with_phase(path, progress, model_bg),
            move |result| Message::FileOpened(open_id, result),
        )
    }
}
```

**Location:** `src/app/update/mod.rs` around line 1323

---

## Step 4: Handle Phase2ProgressTick (app/update/mod.rs)

Add this new handler in the main match statement in `update()`:

```rust
#[cfg(not(target_arch = "wasm32"))]
Message::Phase2ProgressTick => {
    // Poll for events from background loader
    let complete = self.phase2_system.poll_events();
    
    if complete {
        // Loading finished or error occurred
        self.command_line.push_info(
            crate::tf!("File loading complete!").as_ref()
        );
        
        // Close modal
        self.phase2_system.modal.hide();
        
        // Schedule viewport update
        Task::none()
    } else if self.phase2_system.is_loading() {
        // Still loading - queue another tick for next frame
        // This creates smooth progress updates
        Task::perform(
            async_progress_poller(),
            |_| Message::Phase2ProgressTick
        )
    } else {
        Task::none()
    }
}
```

**Location:** `src/app/update/mod.rs` in the main message match

---

## Step 5: Handle Cancel Button (app/update/mod.rs)

Add this handler:

```rust
#[cfg(not(target_arch = "wasm32"))]
Message::Phase2CancelLoading => {
    self.phase2_system.cancel();
    self.command_line.push_info(
        crate::tf!("File loading cancelled.").as_ref()
    );
    Task::none()
}
```

**Location:** `src/app/update/mod.rs` in the main message match

---

## Step 6: Render Progress Modal (app/view/mod.rs)

In your view function, add the progress modal overlay. Find where you render the main UI and add:

```rust
// Render Phase 2 progress modal overlay
#[cfg(not(target_arch = "wasm32"))]
let progress_modal = if self.phase2_system.modal.visible {
    let modal_state = &self.phase2_system.modal.stats;
    
    container(
        column![
            // Modal background
            text("Loading File").size(24),
            
            // Phase indicator
            text(format!("Phase: {}", 
                modal_state.phase
                    .map(|p| p.label())
                    .unwrap_or("...")
            )).size(14),
            
            // Progress bar
            container(
                text(crate::ui::loading_progress::ProgressBar::new(50)
                    .render(modal_state.progress_percent))
            ),
            
            // Progress text
            text(self.phase2_system.progress_text()),
            text(self.phase2_system.eta_text()),
            text(format!("Memory: {} MB", modal_state.memory_mb)),
            
            // Cancel button
            button("Cancel Loading").on_press(Message::Phase2CancelLoading),
        ]
        .spacing(10)
        .padding(20)
    )
    .padding(40)
    .center_x()
    .center_y()
    .into()
} else {
    container(text("")).into()
};

// Overlay the modal on top of viewport
layer(viewport_content)
    .layer(progress_modal)
```

**Location:** `src/app/view/mod.rs` in the view rendering function

---

## Step 7: Helper Function for Progress Polling

Add this helper function to `app/update/mod.rs`:

```rust
/// Async function that yields after a frame to allow progress polling
async fn async_progress_poller() {
    // Use a small delay to let the frame render
    // This ensures smooth progress updates at ~60 FPS
    std::thread::sleep(std::time::Duration::from_millis(16)); // ~60 FPS
}
```

---

## Example: Complete Integration Flow

```rust
// User clicks "Open File"
Message::OpenFile
  ↓
// File picker returns path
Message::OpenPathPicked(Some((path, size)))
  ↓
// Phase 2 loader starts on background thread
self.phase2_system.start_load(path)
  ↓
// Progress modal shows
self.phase2_system.modal.show()
  ↓
// Queue first progress tick
Task::perform(async_progress_poller(), |_| Message::Phase2ProgressTick)
  ↓
// Background thread emits events
LoaderEvent::ParsingProgress { loaded, total }
  ↓
// Progress tick polls for events
Message::Phase2ProgressTick
  ↓
// Poll events and update modal
self.phase2_system.poll_events()
  ↓
// Modal updates
progress_bar → 35%
phase → "Parsing entities..."
eta → "2m 15s remaining"
  ↓
// Render modal in view
progress_modal visible with updates
  ↓
// User sees smooth progress
// (This repeats every ~16ms = 60 FPS)
  ↓
// Loading completes
LoaderEvent::Complete(doc)
  ↓
// Final progress tick
Message::Phase2ProgressTick
  ↓
// Complete flag set
complete = true
  ↓
// Modal closes
self.phase2_system.modal.hide()
  ↓
// Normal UI resumes
```

---

## Testing the Integration

### Quick Test (5 minutes)

```bash
# 1. Build with changes
cargo build

# 2. Run app
./target/debug/opencadstudio

# 3. Try opening test_1k.dxf
# - Should show progress modal
# - File should load quickly
# - Modal should close
```

### Validation Test (15 minutes)

```bash
# 1. Open test_1k.dxf
# - [ ] Modal appears within 100ms
# - [ ] Progress bar shows 0%
# - [ ] Phase shows "Reading file..."

# 2. Wait for completion
# - [ ] Progress bar animates smoothly
# - [ ] Phase changes to "Parsing entities..."
# - [ ] ETA shows reasonable time
# - [ ] Modal closes when complete
# - [ ] File displays correctly

# 3. Try test_10k.dxf
# - [ ] Same flow works
# - [ ] Load time < 2 seconds
# - [ ] Compare vs baseline (should be 5-10x faster)
```

### Performance Test (30 minutes)

```bash
# Measure load time before and after Phase 2

# 1. Measure baseline (if you saved it)
baseline_time = 8.2 seconds

# 2. Measure with Phase 2
time_with_phase2 = ? seconds

# 3. Calculate speedup
speedup = baseline_time / time_with_phase2
# Expected: 5-20x

# 4. Document
echo "Speedup: ${speedup}x"
```

---

## Common Integration Issues

### Issue: Progress modal not showing

**Cause:** Modal rendering not wired to view function  
**Fix:** Make sure `progress_modal` is added to the main container in view()

```rust
// Wrong:
container(main_content).into()

// Right:
layer(container(main_content))
    .layer(progress_modal)
    .into()
```

### Issue: Progress updates stop

**Cause:** Progress tick task not queued  
**Fix:** Ensure `Task::perform(async_progress_poller(), |_| Message::Phase2ProgressTick)` is returned

```rust
// Wrong:
Message::Phase2ProgressTick => {
    self.phase2_system.poll_events();
    Task::none()  // ← This stops the loop!
}

// Right:
Message::Phase2ProgressTick => {
    let complete = self.phase2_system.poll_events();
    if !complete && self.phase2_system.is_loading() {
        Task::perform(async_progress_poller(), |_| Message::Phase2ProgressTick)
    } else {
        Task::none()
    }
}
```

### Issue: Cancellation doesn't work

**Cause:** Cancel message not connected to Phase2 system  
**Fix:** Wire the cancel button to Message::Phase2CancelLoading

```rust
// Wrong:
button("Cancel")  // No .on_press()

// Right:
button("Cancel").on_press(Message::Phase2CancelLoading)
```

### Issue: File doesn't display after loading

**Cause:** Loaded document not added to scene  
**Fix:** In `poll_events()`, when complete, add document to scene

```rust
// In poll_events complete handler:
LoaderEvent::Complete(doc) => {
    // TODO: Add doc to scene
    // self.add_document_to_scene(doc);
    return true;
}
```

---

## Compilation Checklist

After making changes:

```bash
# Check compilation
cargo check

# Expected: No errors

# Run tests (optional)
cargo test --lib

# Expected: All tests pass

# Build release (optional)
cargo build --release

# Expected: Release binary builds successfully
```

---

## Next Steps After Integration

1. **Test with multiple files** (test_1k.dxf through test_25k.dxf)
2. **Measure actual speedup** and document results
3. **Get feedback** from the team
4. **Optimize UI appearance** of progress modal if needed
5. **Profile** to identify remaining bottlenecks (Phase 3B)

---

## Success Criteria

✅ Code compiles without errors  
✅ test_1k.dxf loads and displays  
✅ Progress modal shows and updates  
✅ No UI freezing  
✅ Speedup measured and reported  
✅ No regressions in existing features  

When all ✅, Phase 3A is complete and Phase 3B (Profiling) can begin!
