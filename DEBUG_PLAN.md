# Browser Engine Debugging & Fix Plan

## PART 1: TRIAGING CHECKLIST

Use this systematic checklist to isolate each issue to its root cause:

### Text Spacing Too Large

**Symptom:** Inter-word gaps are exaggerated.

- [ ] **Parsing layer:** Confirm HTML whitespace is being preserved vs. collapsed
  - Where to check: HTML parser's text node creation
  - Log: `text_node.chars().count()` before/after whitespace handling
  - Expected: `"hello  world"` (2 spaces) vs `"hello world"` (1 space)

- [ ] **Layout layer:** Confirm word measurement is reasonable
  - Where to check: `layout_inline_line()` function, `space_width` calculation
  - Current code: `space_width = font_size * 0.3`
  - Test: Font-size 24px → space should be ~7.2px (seems low!)
  - Correct check: Print `word_width` and `space_width` for each word

- [ ] **Font metrics layer:** Confirm space glyph width from font
  - Where to check: Before painting text in `draw_text_glyphs()`
  - Font library API: `font.glyph(' ').scaled(scale).unpositioned().h_metrics().advance_width`
  - Expected: Should match ~25-35% of font-size for most fonts
  - Current issue: Code uses hardcoded `0.3` factor—doesn't ask font for actual space width!

- [ ] **Paint layer:** Confirm x-coordinate advancement during glyph rendering
  - Where to check: `draw_text_glyphs()` after each glyph
  - Current code: `x += glyph.unpositioned().h_metrics().advance_width`
  - Bug risk: If `h_metrics()` is returning incorrect values or code skips spaces during iteration

- [ ] **DPI scaling layer:** Confirm scale factor is applied consistently
  - Where to check: Are `font_size`, `space_width` calculations before or after scale?
  - Current: Layout happens in logical pixels, paint applies `scale_factor` only to position
  - Risk: Font metrics may need scaling too

---

### Text Wrap Wrong (Doesn't Wrap or Wraps Inconsistently)

**Symptom:** Long lines don't break at container edges; or line breaks happen at wrong positions.

- [ ] **Whitespace collapsing:** Confirm `split_whitespace()` isn't losing multiple spaces
  - Current code: `text.split_whitespace().collect()` ← This COLLAPSES all consecutive spaces to single splits
  - Expected: OK for `white-space: normal` (CSS default)
  - Bug: If you ever add `white-space: pre`, this breaks!

- [ ] **Word width calculation:** Confirm width includes all glyphs (not just approximation)
  - Current code: `word.len() as f32 * font_size * 0.55` ← Heuristic!
  - Problem: This doesn't account for:
    - Ligatures (e.g., "fi" in many fonts)
    - Variable-width glyphs (e.g., "w" vs. "i")
    - Kerning (adjustment between specific letter pairs)
  - To verify: Compare heuristic width vs. actual rendered width

- [ ] **Line width limit:** Confirm container width is being passed correctly
  - Current code: `width` parameter in `layout_inline_line()`
  - Check: Print `width` and `x` at start of function
  - Expected: `width` should match effective CSS width (60vw, px, %, etc.)
  - After recent fix: Should be ~720px for 60vw on 1200px viewport

- [ ] **Wrap condition:** Confirm wrap logic is triggered at right moment
  - Current code: `if current_x + word_width > x + width && current_x > x { wrap }`
  - Interpretation: "if word doesn't fit AND we're not at line start, wrap"
  - Edge case bug: What if first word on a line is already wider than container? (→ should still render, overflow)

- [ ] **Leftover space:** Confirm space after last word doesn't cause premature wrap
  - Current code: Adds space after every word except when it exceeds width
  - Potential bug: Space after last word on line may be counted even though it's invisible

---

### Some CSS Not Being Applied

**Symptom:** CSS properties parse but don't appear in layout or paint.

- [ ] **Parsing:** Confirm CSS selectors and declarations are being read
  - Where to check: `browser/src/main.rs`, CSS extraction and parsing logs
  - How: Temporarily add `log()` calls after each CSS parsing stage
  - Expected: Should see `"=== Parsed 4 CSS items from <style> tags ==="` and declarations listed

- [ ] **Selector matching:** Confirm stylesheet is matching DOM nodes
  - Where to check: `engine/src/style/mod.rs`, `compute_style()` function
  - How: Check if selector matching logic exists; what's the current implementation?
  - Expected: `body { width: 60vw }` should match `<body>` tag

- [ ] **Cascade/specificity:** Confirm computed style has the property
  - Where to check: Right before layout, check `style.properties.get("property-name")`
  - How: Add assertion or log before `layout_block_container()`
  - Expected: `width` property should be in HashMap with value `"60vw"`

- [ ] **Layout consumption:** Confirm layout code is reading the CSS property
  - Where to check: `layout_block_container()`, calls to `style.get_width_percentage()`
  - Current code: Has `get_width_percentage()` method—does it work?
  - Expected: Should return `Some(0.6)` for `width: 60vw`

- [ ] **Paint consumption:** Confirm paint code is reading relevant properties
  - Where to check: `draw_box_recursive()` checks `layout.style.get_background_color()`
  - Expected: Properties like `background`, `color`, `font-size` are actually used in paint

---

### Black Artifacts on Resize

**Symptom:** Black regions appear when window is resized; may flicker or persist.

- [ ] **Buffer clearing:** Confirm frame buffer is being cleared before each draw
  - Current code: `for byte in frame.iter_mut() { *byte = 255; }` ← Fills with white (255)
  - Expected: Entire buffer should be 0xFFFFFFFF (white with full alpha) or your background color
  - Test: Print `frame.len()` and number of pixels cleared; compare to expected `width * height * 4`

- [ ] **Resize event handling:** Confirm pixel buffer is recreated with new size
  - Current code: In `Resized` event, creates new `Pixels` and `SurfaceTexture`
  - Risk: If old buffer is still bound or size mismatch, you'll see tears/artifacts
  - Test: Verify `new_size.width` and `new_size.height` match what you expect

- [ ] **Logical vs. physical size mismatch:** Confirm layout width matches buffer size
  - Current code: Layout uses logical pixels (1200), buffer uses physical pixels (2400 on 2x DPI)
  - Recent fix applied scale factor to paint—verify it's applied consistently
  - Risk: If you forget to scale paint coordinates, you'll paint off-screen (→ black region)

- [ ] **Render scheduling:** Confirm redraw is requested after resize
  - Current code: `window.request_redraw()` in Resized event
  - Risk: If event loop is blocked or redraw isn't processed, you'll see stale content
  - Test: Add log message in `RedrawRequested` event to verify it fires after resize

- [ ] **Double buffering:** Confirm pixels.render() is called to present
  - Current code: Calls `pixels.render().unwrap()` after painting
  - Risk: If render fails silently or is called with partial/uninitialized buffer, you see corruption
  - Test: Add `eprintln!()` before and after `pixels.render()`

- [ ] **Partial repaint:** Confirm you're not trying to paint only dirty regions
  - Current code: Clears entire frame and repaints everything
  - This is correct for now; complexity increases if you try fine-grained invalidation

---

## PART 2: TEXT SPACING & WRAPPING FIXES

### Root Causes (Most Common)

1. **Space width not from font metrics** (LIKELY YOUR ISSUE)
   - Using hardcoded factor `0.3` instead of asking font
   - Different fonts have different space widths
   - Fix: Call `font.glyph(' ').scaled(scale).unpositioned().h_metrics().advance_width`

2. **Whitespace collapsing not implemented**
   - Multiple spaces collapse to one (usually desired)
   - Tabs, newlines also collapse
   - But code must be explicit about it

3. **Word width heuristic is imprecise**
   - `len() * font_size * 0.55` doesn't account for variable-width glyphs
   - Ideal: Measure actual glyph advances from font

4. **Space added AFTER word, not BETWEEN**
   - Current code adds space only after word if it fits
   - Edge case: Space at end of line consumes width but may not render
   - Better: Only add space if next word will follow

5. **DPI scaling applied inconsistently**
   - Layout in logical pixels, but space width may be in wrong scale
   - Must scale all font metrics the same way

6. **Line-height calculation missing**
   - Currently `font_size * 1.2` hardcoded
   - Should respect CSS `line-height` property if set

---

### Implementation: Whitespace Collapsing

```rust
// Tokenize text into words + spaces with proper collapsing
fn tokenize_text(text: &str, white_space_mode: &str) -> Vec<TextToken> {
    match white_space_mode {
        "normal" => {
            // Collapse consecutive whitespace to single space
            let normalized = text
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            
            // For now, just split by space (lose punctuation info)
            // Better: use a proper tokenizer that preserves word boundaries
            normalized.split(' ')
                .filter(|s| !s.is_empty())
                .map(|word| TextToken::Word(word.to_string()))
                .intersperse(TextToken::Space)
                .collect()
        },
        "pre" => {
            // Don't collapse; preserve all whitespace exactly
            // (For now, not implemented)
            text.split_whitespace()
                .map(|word| TextToken::Word(word.to_string()))
                .intersperse(TextToken::Space)
                .collect()
        },
        _ => vec![], // Default to normal
    }
}

enum TextToken {
    Word(String),
    Space,
}
```

---

### Implementation: Word Measurement (Including Spaces)

```rust
fn measure_word(font: &Font, word: &str, scale: Scale) -> f32 {
    let mut width = 0.0;
    let mut prev_glyph_id = None;

    for c in word.chars() {
        let g = font.glyph(c);
        let scaled_glyph = g.scaled(scale);
        
        // Add kerning adjustment if available
        if let Some(prev_id) = prev_glyph_id {
            let kern = font.pair_kerning(scale, prev_id, g.id);
            width += kern;
        }
        
        width += scaled_glyph.unpositioned().h_metrics().advance_width;
        prev_glyph_id = Some(g.id);
    }
    
    width
}

fn measure_space(font: &Font, scale: Scale) -> f32 {
    font.glyph(' ')
        .scaled(scale)
        .unpositioned()
        .h_metrics()
        .advance_width
}
```

---

### Implementation: Line Breaking (Greedy Layout)

```rust
fn layout_inline_line_v2(
    dom: &Dom,
    stylesheet: &Stylesheet,
    inline_children: &[NodeId],
    x: f32,
    y: f32,
    max_width: f32,
    font_manager: &mut FontManager,
) -> LayoutBox {
    let mut line_boxes = Vec::new();
    let mut current_x = x;
    let mut line_height = 0.0;
    let mut total_height = 0.0;
    let mut current_y = y;

    for &child_id in inline_children {
        if let NodeType::Text(text) = &dom.nodes[child_id].node_type {
            let style = stylesheet.compute_style(dom, child_id);
            let font_size = style.get_font_size();
            let font_family = style.get_font_family();
            let scale = Scale::uniform(font_size);
            let font = font_manager.load_system_font(font_family)?;
            
            // Tokenize into words
            let words: Vec<&str> = text.split_whitespace().collect();
            
            for (i, word) in words.iter().enumerate() {
                let word_width = measure_word(&font, word, scale);
                let space_width = if i < words.len() - 1 {
                    measure_space(&font, scale)
                } else {
                    0.0  // No space after last word in text node
                };

                // Check: does word fit on current line?
                if current_x + word_width > x + max_width && current_x > x {
                    // No: start new line
                    total_height += line_height;
                    current_y += line_height;
                    current_x = x;
                    line_height = 0.0;
                }

                // Place word box
                let word_box = LayoutBox {
                    dimensions: Dimensions {
                        x: current_x,
                        y: current_y,
                        width: word_width,
                        height: font_size * 1.2,
                    },
                    text_content: Some(word.to_string()),
                    /* ... */
                };
                line_boxes.push(word_box);
                
                current_x += word_width + space_width;
                line_height = line_height.max(font_size * 1.2);
            }
        }
    }

    total_height += line_height;

    LayoutBox {
        dimensions: Dimensions {
            x,
            y,
            width: max_width,
            height: total_height,
        },
        children: line_boxes,
        /* ... */
    }
}
```

---

### Implementation: Letter-Spacing / Word-Spacing Effects

```rust
fn apply_text_spacing(
    width: f32,
    word_spacing: f32,
    letter_spacing: f32,
) -> f32 {
    // width includes natural glyphs + space
    // Add CSS overrides on top
    width + letter_spacing  // Applied per-character
    // Note: word-spacing added per-space separately
}

// In layout:
let base_space_width = measure_space(&font, scale);
let word_spacing = style.get_word_spacing();  // CSS value, in px
let space_width = base_space_width + word_spacing;

let letter_spacing = style.get_letter_spacing();  // CSS value, in px
let letter_offset = letter_spacing;  // Add after each character
```

---

## PART 3: CSS NOT RENDERING PIPELINE

### Minimal "Must-Have" Pipeline

```
┌──────────────┐
│ HTML + Style │
└────────┬─────┘
         │
         ▼
    ┌─────────────────────────────────────────────────────┐
    │ PARSE:                                              │
    │ - Tokenize CSS into rules                           │
    │ - Build selector + declarations                     │
    │ Checkpoint: "Parsed 4 CSS items from <style> tags" │
    └─────────────┬───────────────────────────────────────┘
                  │
                  ▼
    ┌─────────────────────────────────────────────────────┐
    │ STYLE SHEET:                                        │
    │ - Add rules to stylesheet HashMap                   │
    │ - Store by (Selector → Style) mapping               │
    │ Checkpoint: "Stylesheet now has 4 rules"            │
    └─────────────┬───────────────────────────────────────┘
                  │
                  ▼
    ┌─────────────────────────────────────────────────────┐
    │ COMPUTE STYLE:                                      │
    │ - Match DOM node to rules via selector              │
    │ - Merge inherited + element-specific styles         │
    │ Checkpoint: Before layout, assert style.get("prop")│
    │            returns expected value                   │
    └─────────────┬───────────────────────────────────────┘
                  │
                  ▼
    ┌─────────────────────────────────────────────────────┐
    │ LAYOUT:                                             │
    │ - Use computed style to constrain dimensions        │
    │ - Read width, font-size, display, etc.              │
    │ Checkpoint: Print effective_width before/after      │
    │            applying CSS constraint                  │
    └─────────────┬───────────────────────────────────────┘
                  │
                  ▼
    ┌─────────────────────────────────────────────────────┐
    │ PAINT:                                              │
    │ - Read color, background, etc. from style           │
    │ - Draw boxes and text using constraints from layout │
    │ Checkpoint: Verify pixel color matches computed     │
    │            background-color                         │
    └─────────────┴───────────────────────────────────────┘
```

### Instrumentation Points (Example: `color` property)

```rust
// ========== CHECKPOINT 1: PARSE ==========
// In CSS parser, after creating Declaration:
eprintln!("[CSS-PARSE] Decl: {} = {}", decl.property, decl.value);
// Expected: "[CSS-PARSE] Decl: color = red"

// ========== CHECKPOINT 2: STYLESHEET ==========
// In stylesheet.add_rule():
eprintln!("[STYLESHEET] Adding rule {:?} with {} props", selector, style.properties.len());
for (k, v) in &style.properties {
    eprintln!("[STYLESHEET]   {} = {}", k, v);
}
// Expected: "[STYLESHEET]   color = red"

// ========== CHECKPOINT 3: COMPUTE STYLE ==========
// In compute_style(), before returning:
eprintln!("[COMPUTE-STYLE] Node {}: {} -> {:?}", 
    node_id, 
    selector_matched, 
    computed.get("color")
);
// Expected: "[COMPUTE-STYLE] Node 15: true -> Some("red")"

// ========== CHECKPOINT 4: LAYOUT ==========
// In layout_block_container(), using style:
eprintln!("[LAYOUT] Applying color={:?} to layout", style.get_color());
// Expected: "[LAYOUT] Applying color=Some((255, 0, 0)) to layout"

// ========== CHECKPOINT 5: PAINT ==========
// In draw_box_recursive(), when painting:
if let Some((r, g, b)) = layout.style.get_background_color() {
    eprintln!("[PAINT] Drawing bg color RGB({}, {}, {})", r, g, b);
    // ... draw background
}
// Expected: "[PAINT] Drawing bg color RGB(238, 238, 238)"
```

---

## PART 4: RESIZE & BLACK ARTIFACTS STRATEGY

### Root Causes

| Cause | Symptom | Detection |
|-------|---------|-----------|
| **Buffer not cleared** | Black or corrupted last frame | Check: Does `frame.len()` match `width * height * 4`? |
| **Size mismatch** | Partial black region on one side | Check: Does `Pixels` size match `inner_size()`? |
| **Logical/physical confusion** | Scaled/offset artifacts | Check: Are layout pixels and paint scale consistent? |
| **Dangling old buffer** | Torn/split image | Check: Old `SurfaceTexture` dropped before new created? |
| **Render not requested** | Stale frame visible | Check: Does `request_redraw()` happen in Resize handler? |
| **Render loop blocked** | Frame freezes during drag | Check: Is event loop blocking on I/O? |

### Robust Resize Handling

```rust
// === Strategy: Throttle + Immediate Redraw ===

struct ResizeState {
    pending: bool,
    last_size: Option<(u32, u32)>,
}

impl ResizeState {
    fn should_handle_resize(&mut self, new_size: (u32, u32)) -> bool {
        // Ignore if size hasn't actually changed
        if self.last_size == Some(new_size) {
            return false;
        }
        
        // Ignore if resize already pending (throttle)
        if self.pending {
            return false;
        }
        
        self.pending = true;
        self.last_size = Some(new_size);
        true
    }
    
    fn mark_complete(&mut self) {
        self.pending = false;
    }
}

// === In event loop ===

Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
    if resize_state.should_handle_resize((new_size.width, new_size.height)) {
        // Step 1: Verify size
        eprintln!("[RESIZE] Physical size: {}x{}", new_size.width, new_size.height);
        
        // Step 2: Drop old resources explicitly
        drop(pixels);
        
        // Step 3: Recreate with exact same size
        let surface_texture = SurfaceTexture::new(new_size.width, new_size.height, &window);
        pixels = Pixels::new(new_size.width, new_size.height, surface_texture)
            .expect("Failed to create pixels after resize");
        
        eprintln!("[RESIZE] Pixel buffer recreated: {:?}", pixels.frame().len());
        
        // Step 4: Update layout dimensions
        let logical_size = new_size.to_logical(window.scale_factor());
        viewport_width = logical_size.width;
        eprintln!("[RESIZE] Logical viewport: {}x{}", viewport_width, logical_size.height);
        
        // Step 5: Request immediate redraw
        window.request_redraw();
    }
}

Event::RedrawRequested(_) => {
    // Step 1: Layout with current dimensions
    let layout_root = layout_engine.layout_with_viewport(&dom, &stylesheet, viewport_width);
    
    // Step 2: Clear frame completely
    {
        let frame = pixels.frame_mut();
        eprintln!("[PAINT] Clearing {} bytes", frame.len());
        for chunk in frame.chunks_exact_mut(4) {
            chunk[0] = 255;  // R
            chunk[1] = 255;  // G
            chunk[2] = 255;  // B
            chunk[3] = 255;  // A
        }
    }
    
    // Step 3: Paint layout
    draw_layout_and_text(&mut pixels.frame_mut(), &layout_root, /* ... */);
    
    // Step 4: Present to screen
    pixels.render().expect("pixels.render failed");
    eprintln!("[PAINT] Frame rendered");
    
    // Step 5: Mark resize complete
    resize_state.mark_complete();
}

Event::MainEventsCleared => {
    // Continuous redraw to stay responsive
    window.request_redraw();
}
```

### Pseudocode: Full Repaint Invalidation

```rust
struct InvalidationState {
    /// Whether entire frame needs repainting
    dirty: bool,
    /// Optional: dirty rect (not implemented yet)
    dirty_rect: Option<(f32, f32, f32, f32)>,
}

impl InvalidationState {
    fn invalidate_all(&mut self) {
        self.dirty = true;
    }
    
    fn invalidate_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        // For now, treat any dirty rect as full repaint
        self.dirty = true;
    }
    
    fn is_dirty(&self) -> bool {
        self.dirty
    }
    
    fn clear(&mut self) {
        self.dirty = false;
        self.dirty_rect = None;
    }
}

// === On resize ===
invalidation.invalidate_all();
window.request_redraw();

// === On CSS change (future) ===
invalidation.invalidate_all();
window.request_redraw();

// === In redraw handler ===
if invalidation.is_dirty() {
    let frame = pixels.frame_mut();
    // Clear entire frame (for now; could optimize to dirty rect)
    for chunk in frame.chunks_exact_mut(4) {
        chunk[0] = 255;
        chunk[1] = 255;
        chunk[2] = 255;
        chunk[3] = 255;
    }
    
    draw_layout_and_text(frame, &layout_root, /* ... */);
    pixels.render()?;
    
    invalidation.clear();
}
```

---

## PART 5: PRIORITIZED FIXES (Highest ROI First)

### Fix #1: **Space Width From Font Metrics** (HIGHEST ROI)
**Complexity:** Low | **Impact:** Fixes text spacing immediately | **Time:** 10 min

**Current issue:** `space_width = font_size * 0.3` is a guess.  
**Real value:** Should be `font.glyph(' ').scaled(scale).unpositioned().h_metrics().advance_width`

**Implementation:**
```rust
fn measure_space(font: &Font, scale: Scale) -> f32 {
    font.glyph(' ')
        .scaled(scale)
        .unpositioned()
        .h_metrics()
        .advance_width
}

// In layout_inline_line(), replace:
// let space_width = font_size * 0.3;
// with:
let space_width = if let Some(font) = font_manager.load_system_font(style.get_font_family()) {
    measure_space(&font, Scale::uniform(font_size))
} else {
    font_size * 0.3  // Fallback
};
```

---

### Fix #2: **Improve Word Width Measurement** (High ROI)
**Complexity:** Medium | **Impact:** Accurate text wrapping | **Time:** 15 min

**Current issue:** `word_width = word.len() as f32 * font_size * 0.55` is a heuristic.  
**Real value:** Sum of glyph advances from font (accounting for kerning).

**Implementation:**
```rust
fn measure_word(font: &Font, word: &str, scale: Scale) -> f32 {
    let mut width = 0.0;
    let mut prev_glyph_id = None;

    for c in word.chars() {
        let g = font.glyph(c);
        let scaled = g.scaled(scale);
        width += scaled.unpositioned().h_metrics().advance_width;
        prev_glyph_id = Some(g.id);
    }
    
    width
}

// In layout_inline_line(), replace:
// let word_width = word.len() as f32 * font_size * 0.55;
// with:
let font = font_manager.load_system_font(style.get_font_family())?;
let word_width = measure_word(&font, word, Scale::uniform(font_size));
```

---

### Fix #3: **Clear Entire Buffer on Resize** (Medium ROI)
**Complexity:** Low | **Impact:** Stops black artifacts | **Time:** 5 min

**Current issue:** Buffer might not be fully cleared, or size assumptions are wrong.  
**Check:**
```rust
let frame = pixels.frame_mut();
eprintln!("Frame length: {}, expected: {}", frame.len(), width * height * 4);
for byte in frame.iter_mut() {
    *byte = 255;
}
eprintln!("Cleared {} bytes to white", frame.len());
```

---

### Fix #4: **Verify Resize Invalidates Properly** (Medium ROI)
**Complexity:** Low | **Impact:** Ensures redraw happens after resize | **Time:** 10 min

**Current issue:** `request_redraw()` may not be firing, or render is blocked.  
**Check:**
```rust
Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
    eprintln!("[RESIZE-START] Old: {}x{}", pixels.width(), pixels.height());
    eprintln!("[RESIZE-START] New: {}x{}", new_size.width, new_size.height);
    
    // ... recreate buffer ...
    
    eprintln!("[RESIZE-END] Buffer ready");
    window.request_redraw();
}

Event::RedrawRequested(_) => {
    eprintln!("[REDRAW-START]");
    // ... paint ...
    pixels.render()?;
    eprintln!("[REDRAW-END]");
}
```

---

### Fix #5: **Add CSS Property Pipeline Checkpoints** (Medium ROI)
**Complexity:** Low | **Impact:** Isolates CSS failures quickly | **Time:** 20 min

**Add logging at each stage:**
1. CSS parse: `eprintln!("[CSS-PARSE] {} = {}", prop, value)`
2. Add to sheet: `eprintln!("[STYLESHEET] Added rule")`
3. Compute style: `eprintln!("[COMPUTE-STYLE] Node {} matched, style={:?}", node_id, computed_value)`
4. Use in layout: `eprintln!("[LAYOUT] Using {}={:?}", prop, style.get(prop))`
5. Use in paint: `eprintln!("[PAINT] Drawing with {}={:?}", prop, style.get(prop))`

---

### Fix #6: **Whitespace Collapsing (Optional, for Correctness)** (Low ROI)
**Complexity:** Medium | **Impact:** Correct handling of multiple spaces | **Time:** 20 min

**Current:** `split_whitespace()` already collapses multiple spaces (this is correct for `white-space: normal`).  
**Optional:** If you want to preserve spaces for layout analysis, tokenize explicitly.

---

### Fix #7: **Font Fallback Chain** (Low ROI, High Importance Later)
**Complexity:** Low | **Impact:** Prevents missing glyph issues | **Time:** 15 min

**Current:** You have Linux font fallbacks. Ensure all three are tried:
```rust
let fonts_to_try = ["System Font", "Liberation Sans", "DejaVu Sans"];
for family in fonts_to_try {
    if let Some(font) = load_font(family) {
        return Some(font);
    }
}
return Some(default_fallback);
```

---

## Summary Table

| Issue | Root Cause | Fix Priority | Est. Time | Expected Gain |
|-------|-----------|--------------|-----------|----------------|
| Text spacing too large | `space_width = font_size * 0.3` hardcoded | **#1** | 10 min | Visual correctness |
| Text wrapping wrong | Inaccurate heuristic word measurement | **#2** | 15 min | Correct line breaks |
| Black artifacts on resize | Buffer not cleared, or size mismatch | **#3** | 5 min | Stability |
| Resize doesn't trigger redraw | Missing invalidation or blocked event loop | **#4** | 10 min | Responsiveness |
| CSS properties not applied | No pipeline validation | **#5** | 20 min | Debuggability |
| Multiple spaces collapsed | `split_whitespace()` behavior | **#6** | 20 min | Spec compliance |
| Missing glyph fallbacks | Font fallback chain incomplete | **#7** | 15 min | Robustness |

---

## Verification Checklist (After Fixes)

- [ ] Text rendering shows proper inter-word spacing (not exaggerated)
- [ ] Long text wraps at container edge correctly
- [ ] Window resize clears artifacts and redraws cleanly
- [ ] CSS properties (color, background, font-size, width) all apply as expected
- [ ] Adding CSS logging confirms parsing → style → layout → paint pipeline
- [ ] No panics or `.unwrap()` failures in resize path
- [ ] Resizing a 1200px window to 800px and back shows correct layout

---

## Debugging CLI Commands

```bash
# Capture full output including all eprintln! logs
timeout 3 cargo run -p grob_browser 2>&1 | tee output.log

# Search for specific stage
grep "\[LAYOUT\]\|\[PAINT\]" output.log

# Check for panics/errors
grep -i "error\|panic\|failed" output.log

# Watch live output
timeout 10 cargo run -p grob_browser 2>&1 | grep -E "\[PAINT\]|\[RESIZE\]"
```
