# 📖 Browser Engine Documentation Index

## Quick Navigation

### 🎯 **START HERE** → [README_FIXES.md](README_FIXES.md)
Comprehensive overview with problem diagnosis tree, priority matrix, and architecture diagrams.  
**Time to read:** 10 min | **Contains:** Executive summary, diagnosis flow, instrumentation guide

---

## Documentation Files

### 1. [README_FIXES.md](README_FIXES.md) - **Main Entry Point** 
**What:** Executive summary, problem diagnosis tree, visual diagrams  
**For:** Understanding overall strategy and architecture  
**Best for:** First-time reading, quick reference  

```
📌 Key Sections:
├─ Executive Summary (issues & current status)
├─ Problem Diagnosis Tree (flowchart of root causes)
├─ Fix Priority Matrix (what to fix and when)
├─ Architecture: Data Flow (how text spacing works)
├─ Instrumentation Points (where to add logging)
├─ Test Cases (how to verify each fix)
└─ Common Mistakes (anti-patterns to avoid)
```

---

### 2. [DEBUG_PLAN.md](DEBUG_PLAN.md) - **Comprehensive Triaging Guide**
**What:** Detailed step-by-step debugging checklist, instrumentation points, CSS pipeline  
**For:** Investigating root causes, understanding each layer  
**Best for:** Deep dives, when issues arise, layer-by-layer verification  

```
📌 Key Sections:
├─ PART 1: Triaging Checklist
│  ├─ Text spacing too large
│  ├─ Text wrap wrong
│  ├─ Some CSS not being applied
│  └─ Black artifacts on resize
├─ PART 2: Text Spacing & Wrapping Fixes
│  ├─ Root causes (6 common ones)
│  ├─ Whitespace collapsing implementation
│  ├─ Word measurement with fonts
│  └─ Line breaking algorithm
├─ PART 3: CSS Not Rendering Pipeline
│  └─ Minimal must-have pipeline with checkpoints
├─ PART 4: Resize & Black Artifacts Strategy
│  ├─ Root causes table
│  ├─ Robust resize handling
│  └─ Full repaint invalidation
└─ PART 5: Prioritized Fixes
   ├─ Fix #1-7 with complexity/impact
   └─ Verification checklist
```

---

### 3. [QUICK_FIXES.md](QUICK_FIXES.md) - **Action Checklist**
**What:** Step-by-step "copy-paste ready" code changes for each fix  
**For:** Applying fixes one by one  
**Best for:** Implementing remaining fixes, quick reference during coding  

```
📌 Key Sections:
├─ ✓ Fix #1: Space Width (DONE)
├─ Fix #2: Buffer Clearing (TODO - with code)
├─ Fix #3: Verify Resize Redraw (TODO - with code)
├─ Fix #4: CSS Pipeline Instrumentation (TODO - with code)
├─ Fix #5: Word Width Accuracy (Optional)
├─ Build & Test Commands
├─ Visual Testing Checklist
├─ Logging Output Format (what to expect)
└─ Troubleshooting (diagnosis flow chart)
```

---

### 4. [FIXES_APPLIED.md](FIXES_APPLIED.md) - **Change Log**
**What:** Details on what was changed, why, and impact  
**For:** Understanding changes made and next steps  
**Best for:** Reviewing what's been done, context for future work  

```
📌 Key Sections:
├─ ✓ Fix #1: Space Width (Improved 0.3→0.25)
│  ├─ What changed
│  ├─ Files modified
│  ├─ Why this helps
│  └─ Next improvement
├─ Issues Still To Address (priorities #3-5)
├─ Remaining Fixes to Implement (with code snippets)
├─ Testing Checklist (visual verification)
├─ Key Files & Functions (code locations)
├─ Debug Output Inspection (log patterns)
├─ Architecture Overview (diagram)
├─ Common Pitfalls & Solutions
└─ Next Steps (what to do)
```

---

## How to Use This Documentation

### Scenario 1: "I want to understand the whole system"
1. Read [README_FIXES.md](README_FIXES.md) (10 min)
2. Skim [DEBUG_PLAN.md](DEBUG_PLAN.md) sections 1-2 (20 min)
3. Try applying fixes in [QUICK_FIXES.md](QUICK_FIXES.md) (30 min)

**Total: ~1 hour to full understanding + first fixes**

---

### Scenario 2: "Just tell me what to fix next"
1. Jump to [QUICK_FIXES.md](QUICK_FIXES.md) 
2. Find "Fix #2: Buffer Clearing"
3. Copy code, apply to `browser/src/main.rs`
4. Build and test

**Total: ~10 minutes per fix**

---

### Scenario 3: "Text spacing still looks wrong"
1. Open [DEBUG_PLAN.md](DEBUG_PLAN.md)
2. Find "Text Spacing Too Large" section
3. Go through triaging checklist systematically
4. Use instrumentation points to add logging
5. Run and check logs

**Total: 30-60 minutes of investigation**

---

### Scenario 4: "Black regions appear on resize"
1. Open [README_FIXES.md](README_FIXES.md) 
2. Look at "Problem Diagnosis Tree" → resize section
3. Check which possible cause matches your symptom
4. Jump to [QUICK_FIXES.md](QUICK_FIXES.md) → Fix #2 or #3
5. Apply logging and verify

**Total: 15-30 minutes**

---

## Status Summary

| Item | Status | File | Time |
|------|--------|------|------|
| **Issue Analysis** | ✓ Complete | DEBUG_PLAN.md | Done |
| **Fix #1: Space Width** | ✓ Applied | FIXES_APPLIED.md | Done |
| **Fix #2: Buffer Clear** | 📋 Ready | QUICK_FIXES.md | 5 min |
| **Fix #3: Resize Redraw** | 📋 Ready | QUICK_FIXES.md | 5 min |
| **Fix #4: CSS Logging** | 📋 Ready | QUICK_FIXES.md | 10 min |
| **Fix #5: Word Width** | 🔮 Planned | DEBUG_PLAN.md | Future |
| **Fix #6: Line-Height** | 🔮 Planned | - | Future |

---

## Quick Links to Key Sections

### Text Spacing Issues
- **Root cause:** [DEBUG_PLAN.md - Text Spacing Too Large](DEBUG_PLAN.md#text-spacing-too-large)
- **Fix:** [QUICK_FIXES.md - Fix #1](QUICK_FIXES.md#fix-1-space-width-done)
- **Status:** ✓ Fixed (0.3 → 0.25)

### Text Wrapping Issues
- **Root cause:** [DEBUG_PLAN.md - Text Wrap Wrong](DEBUG_PLAN.md#text-wrap-wrong-doesnt-wrap-or-wraps-inconsistently)
- **Fix:** [DEBUG_PLAN.md - Part 2](DEBUG_PLAN.md#part-2-text-spacing--wrapping-fixes)
- **Status:** Improved (width constraint working)

### Black Artifacts on Resize
- **Root cause:** [README_FIXES.md - Problem Diagnosis Tree](README_FIXES.md#problem-diagnosis-tree)
- **Fix:** [QUICK_FIXES.md - Fix #2-3](QUICK_FIXES.md#fix-2-buffer-clearing-on-resize)
- **Status:** 📋 Ready to apply

### CSS Not Applying
- **Root cause:** [DEBUG_PLAN.md - Some CSS Not Being Applied](DEBUG_PLAN.md#some-css-not-being-applied)
- **Fix:** [QUICK_FIXES.md - Fix #4](QUICK_FIXES.md#fix-4-css-pipeline-instrumentation)
- **Status:** 📋 Ready to apply

---

## Code Reference Map

### Layout (Text Spacing & Wrapping)
```
engine/src/layout/mod.rs
├─ LayoutEngine::layout_inline_line()    ← Main wrapping logic
├─ LayoutEngine::measure_word_width()    ← Word measurement (heuristic)
└─ LayoutEngine::get_space_width()       ← Space width (✓ FIXED: 0.25)
```

### Rendering (Pixels & Colors)
```
browser/src/main.rs
├─ Event::RedrawRequested                ← Paint trigger
├─ draw_layout_and_text()                ← Render entry
├─ draw_box_recursive()                  ← Recursive box painter
└─ draw_text_glyphs()                    ← Glyph renderer
```

### Style (CSS Properties)
```
engine/src/style/mod.rs
├─ Style::get_width_percentage()         ← Parse CSS width (60vw → 0.6)
├─ Style::get_font_size()                ← Font size in px
├─ Style::get_background_color()         ← BG color RGB
└─ Style::compute_style()                ← Cascade & matching
```

---

## CLI Commands

### See All Logs
```bash
cd /home/ali-ayyad/Documents/grob
timeout 3 cargo run -p grob_browser 2>&1 | tee all_output.log
```

### Filter for CSS Logs
```bash
grep "\[CSS\]" all_output.log
```

### Filter for Layout Logs
```bash
grep "\[LAYOUT\]" all_output.log
```

### Check Build
```bash
cargo build -q 2>&1 | head -20
```

---

## Next Recommended Action

1. **Review** [README_FIXES.md](README_FIXES.md) to understand overall strategy (10 min)
2. **Apply** Fix #2 from [QUICK_FIXES.md](QUICK_FIXES.md) - buffer clearing (5 min)
3. **Test** by resizing window - check for black regions
4. **Apply** Fix #3 - resize redraw (5 min)
5. **Repeat** for Fixes #4-5

**Estimated time to apply all fixes: 30 minutes**

---

## Notes

- All `.md` files are in the project root: `/home/ali-ayyad/Documents/grob/`
- Actual code files are in: `engine/src/` and `browser/src/`
- Build with: `cargo build -q`
- Test with: `timeout 3 cargo run -p grob_browser 2>&1`
- Check for errors with: `cargo build 2>&1 | grep error`

---

## Questions?

- **"How do I know if my fix worked?"** → See [README_FIXES.md - Test Cases](README_FIXES.md#test-cases)
- **"Where do I add logging?"** → See [README_FIXES.md - Instrumentation Points](README_FIXES.md#instrumentation-points)
- **"What if it breaks?"** → See [README_FIXES.md - Common Mistakes](README_FIXES.md#common-mistakes--how-to-avoid)
- **"Why was Fix #1 needed?"** → See [FIXES_APPLIED.md - Fix #1](FIXES_APPLIED.md#fix-1-space-width-improved-)

---

**Last Updated:** January 21, 2026  
**Status:** 4 documentation files created, 1 fix applied, 3 ready to apply, 2 planned for future

