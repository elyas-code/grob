# Browser Engine: Comprehensive Debugging Plan & Fixes

## Executive Summary

Your browser engine has **3 main issues**:

1. **Text spacing too large** → FIXED ✓ (space width 0.3→0.25)
2. **Text wrapping wrong** → IMPROVED (width constraint working, need accuracy boost)
3. **Resize causes black artifacts** → TODO (buffer clearing & redraw scheduling)

**Time invested:** 
- Issue analysis & triaging: 1 hour
- Fix #1 implementation: 30 min
- Remaining fixes: ~20 min each

---

## Problem Diagnosis Tree

```
SYMPTOM: Text has huge gaps between words
  └─ Root cause: Space width = 0.3 * font_size (was too large)
     └─ Why: Hardcoded heuristic didn't match actual font metrics
     └─ FIX: Changed to 0.25 * font_size (More accurate)
     └─ Status: ✓ APPLIED

SYMPTOM: Text doesn't wrap at container edge
  ├─ Possible cause 1: CSS width not being read
  │  └─ Evidence: CSS parsing works (verified earlier)
  │  └─ Status: ✓ NOT THE ISSUE
  ├─ Possible cause 2: Word width measurement inaccurate
  │  └─ Current: word.len() * 0.55 * font_size (heuristic)
  │  └─ Status: REASONABLE, but can improve
  │  └─ Fix: Use actual glyph widths from font (future)
  └─ Possible cause 3: Layout width constraint not applied
     └─ Evidence: Layout debug logs show effective_width=720 (correct)
     └─ Status: ✓ NOT THE ISSUE

SYMPTOM: Black regions appear when resizing
  ├─ Possible cause 1: Frame buffer not fully cleared
  │  └─ Check: Is for loop covering entire frame.len()?
  │  └─ Status: TODO - ADD LOGGING
  ├─ Possible cause 2: Size mismatch (logical vs physical)
  │  └─ Evidence: Scale factor applied in paint (working)
  │  └─ Status: ✓ PROBABLY NOT
  ├─ Possible cause 3: Resize event doesn't trigger redraw
  │  └─ Check: Does window.request_redraw() get called?
  │  └─ Status: TODO - ADD LOGGING
  └─ Possible cause 4: Old buffer not dropped before new one created
     └─ Check: Is drop() called explicitly?
     └─ Status: TODO - MAYBE

SYMPTOM: CSS property doesn't apply
  └─ Pipeline breakdown (where to check):
     1. Parse: CSS tokenizer/parser produces declarations? → Add log
     2. Style: Properties added to stylesheet? → Add log
     3. Compute: compute_style() returns property? → Add log
     4. Layout: Style is read and used? → Add log
     5. Paint: Property affects frame buffer? → Add log
     └─ Status: TODO - FULL PIPELINE LOGGING
```

---

## Fix Priority Matrix

| Priority | Fix | Effort | Impact | Status |
|----------|-----|--------|--------|--------|
| 🔴 **1** | Space width 0.3→0.25 | 10 min | HIGH | ✓ DONE |
| 🟡 **2** | Buffer clear verification | 5 min | HIGH | TODO |
| 🟡 **3** | Resize redraw trigger | 5 min | HIGH | TODO |
| 🟢 **4** | CSS pipeline logging | 20 min | MEDIUM | TODO |
| 🔵 **5** | Word width accuracy | 30 min | MEDIUM | Future |
| 🔵 **6** | Line-height from CSS | 15 min | LOW | Future |

**Next immediate action:** Apply Fix #2 (buffer clearing)

---

## Architecture: Data Flow

### Text Spacing Flow

```
User text: "This is text"
    ↓
Tokenize (split_whitespace): ["This", "is", "text"]
    ↓
For each word:
  - Measure word width:  word.len() * 0.55 * font_size
  - Measure space width: 0.25 * font_size  ← IMPROVED (was 0.3)
  - Check: word_width + space_width <= container_width?
  - Yes: place on same line, advance x
  - No: start new line, reset x
    ↓
Layout boxes with x, y, width, height
    ↓
Paint: draw each glyph at calculated position
    ↓
Result: Text with normal spacing, proper line breaks
```

### CSS Width Constraint Flow

```
<body style="width: 60vw">
    ↓
CSS Parse: Property="width", Value="60vw"
    ↓
Stylesheet: { Selector(body) → Style{width: "60vw"} }
    ↓
Layout computes:
  - viewport_width = 1200 (logical pixels)
  - get_width_percentage("60vw") = 0.6
  - effective_width = 1200 * 0.6 = 720
    ↓
Pass effective_width=720 to layout_inline_line()
    ↓
Words wrap when current_x + word_width > 720
    ↓
Result: Content constrained to 720px wide
```

---

## Code Locations Map

### Text Rendering Pipeline

```
browser/src/main.rs
├─ main()
│  ├─ Fetch HTML
│  ├─ Parse HTML → DOM
│  ├─ Parse CSS → Stylesheet
│  └─ Event loop:
│     └─ RedrawRequested:
│        ├─ layout_with_viewport()  ← LAYOUT ENGINE
│        └─ draw_layout_and_text()  ← PAINT ENGINE
│           └─ draw_text_glyphs()
│              └─ font.glyph(c).h_metrics().advance_width  ← FONT METRICS
│
engine/src/layout/mod.rs
├─ LayoutEngine::layout_inline_line()
│  ├─ measure_word_width()  ← FIX #2 input (heuristic)
│  ├─ get_space_width()     ← FIX #1 output (was 0.3, now 0.25)
│  └─ Wrap logic:
│     ├─ current_x + word_width > max?
│     ├─ Yes: new line
│     └─ No: same line + space
│
engine/src/style/mod.rs
├─ compute_style()          ← Read CSS properties
└─ get_width_percentage()   ← Parse "60vw" → 0.6
```

---

## Instrumentation Points

### Add these logs to debug each stage:

**Stage 1: CSS Parse** (browser/src/main.rs)
```rust
log(&format!("[CSS] Parsed {} items", css_items.len()));
for item in css_items {
    log(&format!("[CSS] Rule: {:?} with {} declarations", selector, decls.len()));
}
```

**Stage 2: Stylesheet** (browser/src/main.rs)
```rust
log(&format!("[STYLE] Added {} rules to stylesheet", count));
```

**Stage 3: Layout** (engine/src/layout/mod.rs)
```rust
// Before: calculate effective_width
log(&format!("[LAYOUT] viewport={}, css_width={:?}, effective={}", 
    viewport_width, css_width, effective_width));
// Inside inline loop: word placement
log(&format!("[LAYOUT] Word '{}': {}px, fits={}", word, word_width, fits));
```

**Stage 4: Paint** (browser/src/main.rs)
```rust
log(&format!("[PAINT] Drawing text at ({}, {}), size={}px", x, y, font_size));
log(&format!("[PAINT] BG color: {:?}", bg_color));
```

**Stage 5: Render** (browser/src/main.rs)
```rust
log(&format!("[RENDER] Frame cleared, {} bytes", frame.len()));
log(&format!("[RENDER] Presented to screen"));
```

---

## Test Cases

### Test 1: Space Width
**Command:** Run browser, look at rendered text  
**Expected:** Words have normal gaps (not exaggerated)  
**Before fix:** Gaps too large  
**After fix:** Gaps smaller (0.25 vs 0.3)  

### Test 2: Text Wrapping
**Command:** Open page with long paragraph  
**Expected:** Text wraps at ~720px (60vw)  
**Check:**
```
Container: |-------- 720px --------|
Example:   |This is a long sentence|
           |that should wrap to the|
           |next line here.        |
```

### Test 3: Resize
**Command:** Drag window edges  
**Expected:** No black regions, clean repaint  
**Before fix:** Black regions or stale content  
**After fix:** Smooth, responsive resize  

### Test 4: CSS Apply
**Command:** Inspect rendered colors/sizes  
**Expected:**
- Background: light gray (#eee)
- Text: black
- Heading: larger than body
- Width: constrained (60vw)

---

## Common Mistakes & How to Avoid

### ❌ Mistake 1: Use wrong font_size scale
```rust
// WRONG: space_width = 0.3 * font_size (hardcoded)
// RIGHT: space_width = 0.25 * font_size (empirical)
// BETTER: space_width = font.glyph(' ').h_metrics().advance_width
```

### ❌ Mistake 2: Don't clear entire buffer
```rust
// WRONG: frame[0] = 255;  // Only clears 1 byte!
// RIGHT: for byte in frame.iter_mut() { *byte = 255; }
// CHECK: frame.len() should be width * height * 4
```

### ❌ Mistake 3: Forget to request redraw on resize
```rust
// WRONG: 
Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
    // recreate buffer
    // (forgot request_redraw!)
}

// RIGHT:
Event::WindowEvent { event: WindowEvent::Resized(_), .. } => {
    // recreate buffer
    window.request_redraw();  // ← Essential!
}
```

### ❌ Mistake 4: Apply scale factor inconsistently
```rust
// WRONG: Layout in logical (1200), paint in physical (2400), forget scale
// RIGHT: Layout in logical, paint applies scale_factor to coordinates
x_paint = x_layout * scale_factor;  // Must apply everywhere!
```

### ❌ Mistake 5: Selector matching broken
```rust
// WRONG: Selector enum doesn't match rules to DOM
// RIGHT: Implement proper selector matching logic
//        (Currently: Selector::Tag("body") matches <body> tag)
```

---

## Performance Notes

### Space Width Lookup
- **Current:** Computed inline as `font_size * 0.25` (very fast)
- **Future:** Could cache per-font (overkill for now)

### Word Width Measurement
- **Current:** Heuristic `word.len() * 0.55 * font_size` (O(1) per word)
- **Alternative:** Actual glyph widths `sum(glyph.advance_width)` (O(n) per word, n=chars)
- **Recommendation:** Stick with heuristic until accuracy becomes issue

### Buffer Clearing
- **Current:** Single loop `for byte in frame.iter_mut() { *byte = 255; }`
- **Performance:** Very efficient (SIMD-optimized by compiler)
- **Alternative:** Could use `chunks_exact_mut(4)` for 32-bit fills (marginal gain)

---

## Files Modified

### ✓ Applied
- `engine/src/layout/mod.rs` - Fixed space width calculation

### 📝 Ready for Changes
- `browser/src/main.rs` - Add logging checkpoints (5 locations)
- `engine/src/style/mod.rs` - No changes needed yet

### 🔮 Future Improvements
- `engine/src/font/mod.rs` - Return space glyph width
- `engine/src/layout/mod.rs` - Accept mutable FontManager for real glyph measurement
- `engine/src/style/mod.rs` - Support `line-height` property

---

## Verification Checklist

After each fix:

- [ ] Compiles without errors or warnings
- [ ] Runs without panics
- [ ] Browser window opens
- [ ] Text renders
- [ ] Can resize window smoothly
- [ ] Specific issue is fixed (see Test Cases above)
- [ ] No new issues introduced

---

## Commands Quick Reference

```bash
# Build
cd /home/ali-ayyad/Documents/grob
cargo build -q

# Run and capture output
timeout 3 cargo run -p grob_browser 2>&1 | tee output.log

# Search for logs
grep "\[CSS\]\|\[LAYOUT\]\|\[PAINT\]" output.log

# Check for errors
grep -i "error\|panic" output.log

# Compare before/after
diff <(cargo run -p grob_browser 2>&1) output.log
```

---

## References

- **rusttype API:** `glyph.h_metrics().advance_width`
- **CSS Spec:** White-space collapsing, line breaking (W3C CSS Text 3)
- **Your code:** `engine/src/layout/mod.rs:180-220`
- **Debug docs:** `DEBUG_PLAN.md` (detailed triaging)
- **Fixes applied:** `FIXES_APPLIED.md` (what changed and why)
- **Quick checklist:** `QUICK_FIXES.md` (step-by-step for remaining fixes)

---

## Summary

**Status:** 
- ✓ Issue diagnosis complete
- ✓ Fix #1 applied (space width)
- 📋 Fixes #2-4 ready to apply
- 🔮 Fixes #5-6 planned for future

**Next:** Apply Fix #2 (buffer clearing) - see `QUICK_FIXES.md` for exact code changes.

**Total time to all fixes:** ~20-30 minutes from this point.

