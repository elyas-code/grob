# Quick Fix Checklist

**Status:** Fix #1 (space width) complete. Use this guide to apply remaining fixes in order.

---

## ✓ Fix #1: Space Width (DONE)
- Changed from `0.3 * font_size` to `0.25 * font_size`
- File: `engine/src/layout/mod.rs`, method `get_space_width()`
- Impact: Reduced inter-word gaps by ~17%

---

## Fix #2: Buffer Clearing on Resize

**What to do:**  
Add logging to verify buffer is fully cleared before painting.

**File:** `browser/src/main.rs`  
**Function:** Event handler `Event::RedrawRequested`

**Change:**
```rust
Event::RedrawRequested(_) => {
    let layout_root = layout_engine.layout_with_viewport(&dom, &stylesheet, viewport_width);
    
    let frame = pixels.frame_mut();
    let physical_size = window.inner_size();

    // ADD THIS LOG:
    eprintln!("[CLEAR] Frame size: {} bytes ({} x {} pixels)", 
        frame.len(), 
        physical_size.width, 
        physical_size.height
    );

    // Clear frame to white - fill entire buffer
    for byte in frame.iter_mut() {
        *byte = 255;
    }
    
    eprintln!("[CLEAR] Cleared all {} bytes to white", frame.len());

    // ... rest of rendering ...
}
```

**Expected output:**
```
[CLEAR] Frame size: 9600000 bytes (2400 x 1600 pixels)
[CLEAR] Cleared all 9600000 bytes to white
```

**Verify:** After adding logging, resize window and confirm:
1. No black regions appear
2. Log shows correct frame size
3. Text repaints cleanly

---

## Fix #3: Verify Resize Event Triggers Redraw

**File:** `browser/src/main.rs`  
**Function:** Event handler `Event::WindowEvent::Resized`

**Change:**
```rust
Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
    eprintln!("[RESIZE] Resize event: old {}x{} → new {}x{}", 
        pixels.frame().len() / 4 / 1600,  // rough calc
        1600, 
        new_size.width, 
        new_size.height
    );
    
    let logical_size = new_size.to_logical(window.scale_factor());
    viewport_width = logical_size.width;
    eprintln!("[RESIZE] Logical viewport now: {}x{}", viewport_width, logical_size.height);
    
    // Recreate pixels buffer with new dimensions (use physical pixels)
    let surface_texture = SurfaceTexture::new(new_size.width, new_size.height, &window);
    pixels = Pixels::new(new_size.width, new_size.height, surface_texture).unwrap();
    eprintln!("[RESIZE] Pixel buffer recreated");
    
    window.request_redraw();
    eprintln!("[RESIZE] Redraw requested");
}
```

**Expected output during resize:**
```
[RESIZE] Resize event: old 2400x1600 → new 1800x1200
[RESIZE] Logical viewport now: 900x600
[RESIZE] Pixel buffer recreated
[RESIZE] Redraw requested
```

**Verify:** When you resize the window:
1. Logs appear immediately
2. `RedrawRequested` event fires (look for `[CLEAR]` log)
3. No black regions, clean repaint

---

## Fix #4: CSS Pipeline Instrumentation

**File:** `browser/src/main.rs`  
**Location:** After CSS parsing section (around line 80)

**Add these checkpoints:**

### Checkpoint 1: CSS Parse
Already partially done. Add:
```rust
log(&format!("=== Parsed {} CSS items from <style> tags ===", css_items.len()));
```

### Checkpoint 2: Stylesheet Add
In the loop that adds rules:
```rust
for item in css_items {
    if let engine::parser::css::parser::CssItem::Rule(rule) = item {
        log(&format!("[CSS] Processing rule: {:?}", rule.selector));
        let selector = convert_css_selector(&rule.selector);
        
        let mut style = Style::new();
        log(&format!("[CSS]   Found {} declarations", rule.declarations.len()));
        
        for decl in rule.declarations {
            log(&format!("[CSS]     {} = {}", decl.property, decl.value));
            style.properties.insert(decl.property.clone(), decl.value.clone());
        }
        
        stylesheet.add_rule(selector, style);
    }
}
log(&format!("[CSS] Stylesheet now has {} rules", stylesheet.rules.len()));
```

### Checkpoint 3: Layout Consumption
In `layout_with_viewport()` call, add:
```rust
let layout_root = layout_engine.layout_with_viewport(&dom, &stylesheet, viewport_width);
log(&format!("[LAYOUT] Root box width: {}", layout_root.dimensions.width));
```

### Checkpoint 4: Paint Consumption
In `draw_layout_and_text()`, add before painting:
```rust
if let Some((r, g, b)) = layout.style.get_background_color() {
    log(&format!("[PAINT] BG color: RGB({},{},{})", r, g, b));
}
if let Some(color) = layout.style.get("color") {
    log(&format!("[PAINT] Text color: {}", color));
}
```

**Verify:** With these checkpoints, you can trace any CSS property through the entire pipeline:
1. Parse: "Parsed X items"
2. Stylesheet: "Rule matched, Y declarations"
3. Layout: "Width constraint applied"
4. Paint: "Drawing with color Z"

---

## Fix #5: Word Width Accuracy (Optional, Future)

**Current:** Heuristic `word.len() * 0.55 * font_size`  
**Future:** Measure actual glyph widths from font

**Blocker:** FontManager methods require `&mut self`, but layout only has `&self`

**Solution (pseudocode):**
```rust
// Option 1: Make layout take &mut FontManager (breaks current signature)
// Option 2: Create static/thread-local font cache
// Option 3: Pre-compute word widths in a preprocessing pass
// Option 4: Use rusttype directly in layout (avoids FontManager)
```

**For now:** Current heuristic is reasonable. Revisit if text wrapping is noticeably wrong.

---

## Build & Test

```bash
cd /home/ali-ayyad/Documents/grob

# Build with all fixes
cargo build -q 2>&1

# Run and observe for 3 seconds
timeout 3 cargo run -p grob_browser 2>&1 | tee last_run.log

# Check for specific logs
grep "\[CLEAR\]\|\[RESIZE\]\|\[CSS\]" last_run.log

# Check for errors/panics
grep -i "error\|panic\|failed" last_run.log
```

---

## Visual Testing Checklist

After applying fixes:

1. **Text Spacing**
   - [ ] Inter-word gaps look normal (not huge)
   - [ ] Multiple words on same line are readable
   - [ ] Space width seems proportional to font size

2. **Text Wrapping**
   - [ ] Long paragraphs wrap at ~720px (60vw)
   - [ ] Words don't get cut off mid-line
   - [ ] Line breaks happen at word boundaries

3. **Resize Behavior**
   - [ ] Can drag window edges without freezing
   - [ ] No black regions appear during resize
   - [ ] Content repaints cleanly
   - [ ] Layout adapts to new size

4. **CSS Application**
   - [ ] Background color is light gray (#eee) ✓
   - [ ] Text is black by default ✓
   - [ ] Heading sizes differ from body ✓
   - [ ] Page width is constrained (not full screen)

---

## Logging Output Format

Expected logs show this flow:

```
Fetching HTML from: https://example.com
Successfully fetched 513 bytes
Token 0: Doctype ...
Token 1: StartTag ...
...
=== Parsed 4 CSS items from <style> tags ===
[CSS] Processing rule: Element("body")
[CSS]   Found 4 declarations
[CSS]     width = 60vw
[CSS]   ... more properties ...
[LAYOUT] Root box width: 720
[RESIZE] Resize event: 1200x800 → 1000x600
[RESIZE] Logical viewport: 500x300
[CLEAR] Frame size: 3000000 bytes (1000 x 750 pixels)
[CLEAR] Cleared all 3000000 bytes to white
[PAINT] BG color: RGB(238, 238, 238)
```

---

## Troubleshooting

### Symptom: Text still has huge gaps
**Diagnosis:** Space width not updated  
**Check:** Confirm `get_space_width()` returns `font_size * 0.25` not `0.3`  
**File:** `engine/src/layout/mod.rs` line ~45

### Symptom: Black region after resize
**Diagnosis:** Buffer not cleared or size mismatch  
**Check:**
- Is buffer fully cleared? (`[CLEAR]` log shows full frame size)
- Does `new_size` match actual window size? (Compare to visual)
- Is `request_redraw()` called? (`[RESIZE]` log should appear)

### Symptom: CSS properties don't apply
**Diagnosis:** Property not reaching paint  
**Check:**
- CSS parsed? (`[CSS]` logs show property)
- Selector matched? (`[LAYOUT]` or `[PAINT]` logs)
- Property getter exists? (`style.get_background_color()` method)

### Symptom: Panics or `.unwrap()` failures
**Diagnosis:** API contract violated  
**Check:**
- Building without errors? (`cargo build -q`)
- Running without flags? (No `--release` needed for debugging)
- Font loading works? (Text renders at all)

---

## Summary

| Fix | Status | Impact | Time |
|-----|--------|--------|------|
| #1: Space width | ✓ DONE | -17% gap size | Done |
| #2: Buffer clear | TODO | No black artifacts | 5 min |
| #3: Resize redraw | TODO | Responsive window | 5 min |
| #4: CSS logging | TODO | Better debugging | 10 min |
| #5: Word width | Optional | Exact wrapping | Future |

**Total time to apply all remaining fixes: ~20 minutes**

---

## Questions?

Refer to:
- **DEBUG_PLAN.md** - Comprehensive triaging and architecture
- **FIXES_APPLIED.md** - Details on Fix #1 and rationale
- **engine/src/layout/mod.rs** - Layout algorithm and width calculations
- **browser/src/main.rs** - Rendering, event loop, CSS loading

