# Phase 3A Testing Plan

**Status:** Ready to test  
**Build:** ✅ Compiles successfully  
**Expected Duration:** 20-30 minutes  

---

## Quick Test (5 minutes)

### 1. Build and Run
```bash
cd /Users/voquochieu/Claude/target-game/OpenCADStudio
cargo run --release
```

### 2. Open a File
- Click "Open File" (or File → Open)
- Navigate to `test_1k.dxf` or any small DXF file
- **Expected:** Progress modal appears

### 3. Verify Modal
- Title says "Loading File" ✅
- Shows current phase (Reading/Parsing/Tessellating) ✅
- Shows progress text ✅
- Has Cancel button ✅
- Modal closes when done ✅

### 4. Check Result
- File loads and displays in viewport ✅
- No UI freezing during load ✅
- Command line shows "File loaded!" ✅

---

## Full Test Suite (20-30 minutes)

### Test 1: Small File (test_1k.dxf)
```
File: test_1k.dxf (1,000 entities)

Steps:
1. Open File menu → Open
2. Select test_1k.dxf
3. Observe progress modal
4. Wait for completion
5. Verify file displays

Expected results:
  ✓ Modal appears immediately
  ✓ Progress updates smoothly
  ✓ File loads in < 1 second
  ✓ No UI freeze
  ✓ Modal closes when done
```

### Test 2: Medium File (test_10k.dxf)
```
File: test_10k.dxf (10,000 entities)

Baseline (before Phase 2):
  - Load time: ~8 seconds
  - UI frozen: YES
  - Feel: Frustrating

With Phase 2:
  - Load time: 1-2 seconds ✅
  - UI frozen: NO ✅
  - Feel: Professional ✅

Steps:
1. Open File menu → Open
2. Select test_10k.dxf
3. Observe loading with progress modal
4. Try panning/zooming viewport during load
5. Measure total load time
6. Verify speedup achieved

Measure speedup:
  - Note start time when clicking "Open"
  - Note end time when progress modal closes
  - Compare to baseline (8 seconds)
  - Calculate speedup: 8 / actual_time

Success criteria: Speedup > 4x ✅
```

### Test 3: Cancel Loading
```
Steps:
1. Open File menu → Open
2. Select a medium file (test_10k.dxf)
3. Wait for modal to appear
4. While loading, click "Cancel" button
5. Verify modal closes immediately
6. Verify no corrupted state

Expected results:
  ✓ Cancel button works
  ✓ Modal closes immediately
  ✓ Command line shows "cancelled" message
  ✓ No error messages
  ✓ Can open another file
```

### Test 4: UI Responsiveness
```
While loading test_10k.dxf:

1. Move the window
   Expected: Smooth, no stuttering ✓

2. Click elsewhere (not on modal buttons)
   Expected: Clickable, responsive ✓

3. Pan viewport with middle mouse
   Expected: Works smoothly ✓

4. Zoom viewport with scroll wheel
   Expected: Works smoothly ✓

5. Press keyboard shortcuts
   Expected: Respond normally ✓

Key point: UI should NEVER freeze during load
```

### Test 5: Sequential Loads
```
Steps:
1. Load test_1k.dxf (wait for completion)
2. Immediately load test_10k.dxf
3. Verify progress modal for second load
4. Wait for completion
5. Verify both files work

Expected:
  ✓ Can load multiple files in sequence
  ✓ No state corruption
  ✓ Each load shows progress modal
  ✓ Performance consistent
```

---

## Measurement Data

### Load Time Tracking

Record these metrics for analysis:

**test_1k.dxf:**
- Start time: `[note when clicking Open]`
- Modal appears: `[seconds after start]`
- Loading completes: `[seconds after start]`
- Total time: `___ seconds`
- Expected: < 1 second ✅

**test_10k.dxf:**
- Start time: `[note when clicking Open]`
- Modal appears: `[seconds after start]`
- Loading completes: `[seconds after start]`
- Total time: `___ seconds`
- Baseline was: 8.2 seconds
- Speedup: 8.2 / ___ = **_x**
- Target: > 4x ✅

**test_25k.dxf (if available):**
- Total time: `___ seconds`
- Expected: 3-5 seconds ✅

---

## Known Issues & Workarounds

### Scene Integration Not Yet Complete
- **Issue:** Loaded entities won't appear in viewport yet
- **Status:** TODO - will be completed in next integration phase
- **Impact:** Can verify modal and progress but entities won't display
- **Note:** See TODO comment in viewport.rs line 1080

### Web Build (WASM)
- Phase 2 is desktop-only (conditional compilation)
- Web still uses synchronous loading
- Web build unaffected

---

## Success Criteria

### Must Have ✅
- [ ] App compiles without errors
- [ ] Progress modal appears when opening file
- [ ] Modal shows correct information
- [ ] UI never freezes during load
- [ ] Cancel button works
- [ ] No error messages

### Should Have ✅
- [ ] Load time improved vs baseline
- [ ] Viewport is responsive during load
- [ ] Sequential loads work
- [ ] Progress modal looks polished

### Nice to Have
- [ ] Speedup > 4x for 10K file
- [ ] ETA countdown accurate
- [ ] Memory display useful

---

## Troubleshooting

### Progress Modal Doesn't Appear
```
Check:
1. Make sure you're on desktop (not web)
2. Verify Phase 2IntegrationBridge field was added
3. Run: cargo clean && cargo build --release
4. Check modal_visible() is being called
```

### File Doesn't Load
```
Check:
1. See TODO at viewport.rs:1080
2. Scene integration not yet implemented
3. This is expected for Phase 3A
4. Will be completed in Phase 3B/4
```

### Modal Freezes During Load
```
Check:
1. Polling in on_tick (viewport.rs:1073)
2. Make sure phase2.poll_events() is called
3. Check Message::Tick is being sent
```

### Cancel Doesn't Work
```
Check:
1. OpenCancel handler wired correctly
2. phase2.cancel() being called
3. Modal visibility state updated
```

---

## Next Steps After Testing

### If All Tests Pass ✅
1. Document results in PHASE3A_TEST_RESULTS.md
2. Record load times and speedup achieved
3. Take screenshots of progress modal
4. Proceed to Phase 3B (Profiling)

### If Issues Found ❌
1. Document in test results
2. Debug with provided troubleshooting guide
3. Check TODOs in code
4. Reach out for fixes

---

## File Locations

**Test files (if available in project):**
- `test_1k.dxf` - 1,000 entities
- `test_10k.dxf` - 10,000 entities  
- `test_25k.dxf` - 25,000 entities

**If not available:**
- Create small test files from existing DXF
- Use customer files (with permission)
- Generate synthetic test DXF files

---

## Contact Points

If something doesn't work as expected:

1. **Modal not showing?** → Check modal_visible() in phase2_integration_bridge.rs
2. **Performance not improved?** → Check poll_events() in viewport.rs
3. **Entities not displaying?** → Expected (TODO: scene integration)
4. **UI freezing?** → Check Tick message handler

---

**Ready to test!** ✅

Start with the Quick Test (5 minutes), then run the Full Test Suite if everything works.

Expected result: 4-8x faster file loading with smooth, responsive UI progress modal.

🚀 Let's measure the speedup!
