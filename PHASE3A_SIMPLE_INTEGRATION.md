# Phase 3A: Simplified Integration Guide

**Goal:** Wire Phase 2 into your app in 2-3 hours  
**Approach:** Using Phase2IntegrationBridge (cleaner, easier)  
**Status:** Module created ✅, compiling ✅

---

## Quick Start (3 Simple Steps)

### Step 1: Add Field to App Struct (5 minutes)

In your `src/app/mod.rs`, find the `pub struct App { ... }` definition and add:

```rust
pub struct App {
    // ... your existing fields ...
    
    // Phase 2: Background file loading (add this)
    #[cfg(not(target_arch = "wasm32"))]
    phase2: crate::app::phase2_integration_bridge::Phase2IntegrationBridge,
    
    // ... rest of fields ...
}
```

In the `impl App::new()` constructor, add:

```rust
impl App {
    pub fn new(...) -> Self {
        Self {
            // ... existing field initialization ...
            
            #[cfg(not(target_arch = "wasm32"))]
            phase2: crate::app::phase2_integration_bridge::Phase2IntegrationBridge::new(),
            
            // ... rest of initialization ...
        }
    }
}
```

### Step 2: Handle File Loading (30 minutes)

In `src/app/update/mod.rs`, find the `Message::OpenPathPicked` handler and modify it:

```rust
Message::OpenPathPicked(Some((path, size_bytes))) => {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // Use Phase 2 background loader
        self.phase2.start_load(path.clone());
        
        // Return immediately - loading happens in background
        // We'll poll for events on each frame tick
        return Task::none();
    }
    
    #[cfg(target_arch = "wasm32")]
    {
        // Web: use existing synchronous loading
        // (keep your existing code here)
    }
}
```

Add a handler for progress ticks in the `Message` match statement:

```rust
#[cfg(not(target_arch = "wasm32"))]
Message::Tick(_instant) => {
    // Poll for Phase 2 loading events
    if self.phase2.is_loading() {
        let complete = self.phase2.poll_events();
        
        if complete {
            // Loading finished
            // TODO: Add document to scene here
            self.command_line.push_info(
                crate::tf!("File loaded!").as_ref()
            );
        }
    }
    
    // ... rest of your Tick handling ...
}
```

### Step 3: Render Progress Modal (1 hour)

In `src/app/view/mod.rs`, in your main view function, add the progress modal overlay:

```rust
// After your main content container
#[cfg(not(target_arch = "wasm32"))]
let progress_overlay = if self.phase2.modal_visible() {
    container(
        column![
            // Title
            text("Loading File").size(24).bold(),
            
            // Phase indicator
            text(format!(
                "Phase: {}",
                self.phase2
                    .current_phase()
                    .map(|p| p.label())
                    .unwrap_or("...")
            )).size(14),
            
            // Progress bar
            container(
                text(crate::ui::loading_progress::ProgressBar::new(50)
                    .render(self.phase2.progress_percent()))
                .monospace()
            ),
            
            // Progress details
            text(self.phase2.progress_text()).size(12),
            text(self.phase2.eta_text()).size(12),
            
            // Cancel button
            button("Cancel")
                .on_press(Message::OpenCancel),
        ]
        .spacing(15)
        .padding(20)
    )
    .padding(40)
    .center_x()
    .center_y()
    .into()
} else {
    container(text("")).into()
};

// Layer the progress modal on top of your viewport
layer(main_viewport_content)
    .layer(progress_overlay)
```

---

## Testing (30 minutes)

### Test 1: File Opens
```bash
1. Build: cargo build
2. Run app
3. Click "Open File"
4. Select test_1k.dxf
5. Verify:
   - Modal appears
   - Progress bar shows
   - File loads quickly
   - Modal closes
```

### Test 2: Measure Speedup
```bash
1. Time the load with test_10k.dxf
2. Baseline should be: ~8 seconds
3. With Phase 2 should be: ~1-2 seconds
4. Expected speedup: 4-8x ✅
```

### Test 3: UI Responsiveness
```bash
1. During load, try:
   - Moving the window
   - Clicking elsewhere
   - Panning viewport
2. Verify: NO FREEZING ✅
```

---

## Key Points

### What the Bridge Does
- **Manages background loading** without UI blocking
- **Tracks progress** (phase, percentage, ETA)
- **Batches entities** to reduce geometry bumps
- **Handles cancellation** gracefully

### What You Need to Provide
1. **Field in App struct** - where to store the bridge
2. **Tick event handler** - to poll for progress
3. **UI rendering** - to show progress modal
4. **Scene integration** - to add loaded entities (TODO)

### What Compiles Already ✅
- All Phase 2 modules
- Integration bridge
- Test infrastructure

---

## File Reference

**Create/Modify:**
- `src/app/mod.rs` - Add phase2 field
- `src/app/update/mod.rs` - Handle OpenPathPicked + Tick
- `src/app/view/mod.rs` - Render progress modal

**Already Exists:**
- `src/app/phase2_integration_bridge.rs` ✅
- All Phase 2 modules ✅

---

## Troubleshooting

### Modal doesn't show
→ Check that `layer(content).layer(modal)` is in your view

### Progress doesn't update
→ Make sure you're polling in `Message::Tick` handler

### Loading doesn't start
→ Verify `phase2.start_load(path)` is called in OpenPathPicked

### Compilation error
→ Run `cargo check --lib` to see exact error

---

## Estimated Timeline

| Step | Time | Status |
|------|------|--------|
| Add field to App | 5 min | 📝 Ready |
| Handle OpenPathPicked | 15 min | 📝 Ready |
| Add Tick handler | 10 min | 📝 Ready |
| Render modal | 30 min | 📝 Ready |
| Test & measure | 30 min | 📝 Ready |
| **Total** | **90 min** | ✅ |

---

## Success Criteria

After completing these steps:

✅ App compiles without errors  
✅ File dialog works and starts Phase 2 loading  
✅ Progress modal appears and updates  
✅ test_1k.dxf loads in < 500ms  
✅ No UI freezing during load  
✅ Speedup measured and documented  

---

## Next After Integration

Once Phase 3A is complete:

1. **Phase 3B:** Profile the loading process
   - Run flamegraph to identify next bottleneck
   - Document findings
   - Decide: Phase 4 (blocks) or Phase 5 (quadtree)

2. **Then:** Implement remaining phases (Phase 4-5)
   - Each will follow similar integration pattern
   - Each will add more speedup

---

## Questions?

Refer to:
- `PHASE3A_INTEGRATION_CODE.md` - Detailed code examples
- `phase2_integration_bridge.rs` - Implementation details
- `MASTER_OPTIMIZATION_ROADMAP.md` - Big picture strategy

---

**Ready to integrate?**

Start with Step 1: Add the field to App struct.

Should take ~90 minutes total. Let's go! 🚀
