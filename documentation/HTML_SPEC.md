# HTML Implementation Roadmap

## WHATWG HTML Living Standard Alignment

This document tracks the implementation status of HTML parsing according to the
[WHATWG HTML Living Standard](https://html.spec.whatwg.org/multipage/).

### Implementation Status Legend
- ✅ **Implemented**: Fully compliant with spec
- ⚠️ **Partial**: Some aspects implemented, deviations noted
- ❌ **Not Implemented**: Feature not yet supported
- 🚧 **In Progress**: Currently being implemented

---

## 1. Tokenization (Spec Section 13.2.5)

### Tokenizer States

| State | Status | Notes |
|-------|--------|-------|
| Data state | ⚠️ Partial | Basic text/tag detection, no entity handling |
| RCDATA state | ❌ | Not implemented |
| RAWTEXT state | ❌ | Not implemented |
| Script data state | ❌ | Not implemented |
| PLAINTEXT state | ❌ | Not implemented |
| Tag open state | ⚠️ Partial | Basic tag parsing |
| End tag open state | ⚠️ Partial | Basic end tag parsing |
| Tag name state | ⚠️ Partial | Basic name extraction |
| RCDATA less-than sign state | ❌ | Not implemented |
| Before attribute name state | ⚠️ Partial | Basic whitespace handling |
| Attribute name state | ⚠️ Partial | Basic attribute parsing |
| After attribute name state | ⚠️ Partial | Basic handling |
| Before attribute value state | ⚠️ Partial | Basic handling |
| Attribute value (quoted) states | ⚠️ Partial | Double/single quotes supported |
| Attribute value (unquoted) state | ⚠️ Partial | Basic support |
| Comment states | ⚠️ Partial | Basic <!-- --> support |
| DOCTYPE states | ⚠️ Partial | Basic DOCTYPE detection |
| Character reference states | ❌ | Not implemented |

### Token Types

| Token Type | Status | Notes |
|------------|--------|-------|
| DOCTYPE | ⚠️ Partial | Name only, no public/system ID |
| Start tag | ✅ | With attributes, self-closing |
| End tag | ✅ | Basic support |
| Comment | ⚠️ Partial | Basic support |
| Character (text) | ⚠️ Partial | Whitespace normalization issues |
| End-of-file | ✅ | Implemented |

---

## 2. Tree Construction (Spec Section 13.2.6)

### Insertion Modes

| Mode | Status | Notes |
|------|--------|-------|
| initial | ⚠️ Partial | Basic DOCTYPE handling |
| before html | ⚠️ Partial | Auto-creates html element |
| before head | ⚠️ Partial | Auto-creates head element |
| in head | ⚠️ Partial | Basic head element handling |
| in head noscript | ❌ | Not implemented |
| after head | ⚠️ Partial | Transitions to in body |
| in body | ⚠️ Partial | Most complex mode, partial |
| text | ❌ | Not implemented |
| in table | ❌ | Not implemented |
| in table text | ❌ | Not implemented |
| in caption | ❌ | Not implemented |
| in column group | ❌ | Not implemented |
| in table body | ❌ | Not implemented |
| in row | ❌ | Not implemented |
| in cell | ❌ | Not implemented |
| in select | ❌ | Not implemented |
| in select in table | ❌ | Not implemented |
| in template | ❌ | Not implemented |
| after body | ❌ | Not implemented |
| in frameset | ❌ | Not implemented |
| after frameset | ❌ | Not implemented |
| after after body | ❌ | Not implemented |
| after after frameset | ❌ | Not implemented |

### Algorithm Steps

| Feature | Status | Notes |
|---------|--------|-------|
| Stack of open elements | ✅ | Basic stack implementation |
| Active formatting elements | ❌ | Not implemented |
| Foster parenting | ❌ | Not implemented |
| Adoption agency algorithm | ❌ | Not implemented |
| Generic RCDATA/RAWTEXT parsing | ❌ | Not implemented |
| Reset insertion mode | ❌ | Not implemented |
| Token reprocessing | ❌ | Not implemented |

---

## 3. DOM Construction (Spec Section 4)

### Node Types

| Type | Status | Notes |
|------|--------|-------|
| Document | ❌ | Using "document" element instead |
| Element | ✅ | Basic implementation |
| Text | ✅ | Basic implementation |
| Comment | ❌ | Parsed but not stored in DOM |
| DocumentType | ❌ | Not implemented |
| DocumentFragment | ❌ | Not implemented |
| Attr | ⚠️ Partial | Stored as Vec<(String, String)> |

### Element Categories

| Category | Status | Elements |
|----------|--------|----------|
| Void elements | ✅ | area, base, br, col, embed, hr, img, input, link, meta, param, source, track, wbr |
| Raw text elements | ❌ | script, style |
| Escapable raw text | ❌ | textarea, title |
| Foreign elements | ❌ | SVG, MathML |
| Normal elements | ⚠️ Partial | All others |

---

## 4. Element Support

### Document Structure

| Element | Status | Spec Reference | Notes |
|---------|--------|----------------|-------|
| html | ⚠️ Partial | 4.1.1 | Created automatically |
| head | ⚠️ Partial | 4.2.1 | Created automatically |
| body | ⚠️ Partial | 4.3.1 | Created automatically |
| title | ⚠️ Partial | 4.2.2 | Parsed, text extracted |

### Sections

| Element | Status | Spec Reference | Notes |
|---------|--------|----------------|-------|
| article | ⚠️ Partial | 4.3.2 | Treated as block |
| section | ⚠️ Partial | 4.3.3 | Treated as block |
| nav | ⚠️ Partial | 4.3.4 | Treated as block |
| aside | ⚠️ Partial | 4.3.5 | Treated as block |
| h1-h6 | ⚠️ Partial | 4.3.6 | Treated as block with styles |
| header | ⚠️ Partial | 4.3.8 | Treated as block |
| footer | ⚠️ Partial | 4.3.9 | Treated as block |
| address | ⚠️ Partial | 4.3.10 | Treated as block |

### Grouping Content

| Element | Status | Spec Reference | Notes |
|---------|--------|----------------|-------|
| p | ⚠️ Partial | 4.4.1 | Auto-closing, block |
| hr | ⚠️ Partial | 4.4.2 | Void element |
| pre | ⚠️ Partial | 4.4.3 | Whitespace not preserved |
| blockquote | ⚠️ Partial | 4.4.4 | Treated as block |
| ol | ⚠️ Partial | 4.4.5 | Basic list support |
| ul | ⚠️ Partial | 4.4.6 | Basic list support |
| li | ⚠️ Partial | 4.4.8 | Auto-closing |
| dl | ⚠️ Partial | 4.4.9 | Treated as block |
| dt | ⚠️ Partial | 4.4.10 | Auto-closing |
| dd | ⚠️ Partial | 4.4.11 | Auto-closing |
| div | ⚠️ Partial | 4.4.15 | Treated as block |

### Text-Level Semantics

| Element | Status | Spec Reference | Notes |
|---------|--------|----------------|-------|
| a | ⚠️ Partial | 4.5.1 | href extraction, styling |
| em | ⚠️ Partial | 4.5.2 | Italic styling |
| strong | ⚠️ Partial | 4.5.3 | Bold styling |
| small | ⚠️ Partial | 4.5.4 | No special handling |
| s | ⚠️ Partial | 4.5.5 | Strikethrough |
| cite | ⚠️ Partial | 4.5.6 | Italic styling |
| q | ❌ | 4.5.7 | Not implemented |
| dfn | ⚠️ Partial | 4.5.8 | No special handling |
| abbr | ⚠️ Partial | 4.5.9 | No special handling |
| code | ⚠️ Partial | 4.5.15 | Monospace font |
| var | ⚠️ Partial | 4.5.16 | Italic styling |
| samp | ⚠️ Partial | 4.5.17 | Monospace font |
| kbd | ⚠️ Partial | 4.5.18 | Monospace font |
| sub | ❌ | 4.5.19 | Not implemented |
| sup | ❌ | 4.5.20 | Not implemented |
| i | ⚠️ Partial | 4.5.21 | Italic styling |
| b | ⚠️ Partial | 4.5.22 | Bold styling |
| u | ⚠️ Partial | 4.5.23 | Underline styling |
| mark | ⚠️ Partial | 4.5.24 | No special handling |
| span | ⚠️ Partial | 4.5.26 | Inline container |
| br | ✅ | 4.5.27 | Void element |

### Embedded Content

| Element | Status | Spec Reference | Notes |
|---------|--------|----------------|-------|
| img | ⚠️ Partial | 4.8.3 | src/alt, basic rendering |
| iframe | ❌ | 4.8.5 | Not implemented |
| embed | ❌ | 4.8.6 | Not implemented |
| object | ❌ | 4.8.7 | Not implemented |
| video | ❌ | 4.8.9 | Not implemented |
| audio | ❌ | 4.8.10 | Not implemented |
| canvas | ❌ | 4.12.5 | Not implemented |

### Forms

| Element | Status | Notes |
|---------|--------|-------|
| form, input, button, etc. | ❌ | Not implemented |

### Tables

| Element | Status | Notes |
|---------|--------|-------|
| table, tr, td, th, etc. | ❌ | Not implemented |

---

## 5. Known Deviations from Spec

### Critical Deviations

1. **No proper tokenizer state machine**: Current tokenizer is simplified, doesn't follow all state transitions
2. **No character reference parsing**: Entities like `&amp;` not decoded
3. **No active formatting elements**: Formatting element adoption not implemented
4. **No foster parenting**: Misnested table content not handled correctly
5. **No token reprocessing**: Tokens consumed once, not reprocessed

### Minor Deviations

1. **Text node merging**: Adjacent text nodes may not be merged
2. **Whitespace handling**: Inter-element whitespace not always correct
3. **Error recovery**: Parse errors not handled per spec

---

## 6. Implementation Priority

### Phase 1 (Current Focus)
1. ✅ Basic tokenization
2. ✅ Basic tree construction
3. 🚧 Proper document structure (html/head/body)
4. 🚧 Text node handling
5. 🚧 Anchor semantics

### Phase 2
1. Block vs inline flow
2. Basic phrasing content
3. Lists (ul, ol, li)

### Phase 3
1. Tables
2. Forms (display only)
3. Better error recovery

### Phase 4
1. Character references
2. Active formatting elements
3. Foster parenting

---

## 8. CSS Default Styles (User Agent Stylesheet)

The following table documents the default CSS styles applied by the browser's user agent stylesheet
per the HTML specification. These are automatically applied to HTML elements without explicit styling.

### Default Element Margins and Styling

| Element | Font Size | Font Weight | Margin | Padding | Notes |
|---------|-----------|-------------|--------|---------|-------|
| h1 | 2em | bold | 0.67em 0 | — | Heading level 1 |
| h2 | 1.5em | bold | 0.75em 0 | — | Heading level 2 |
| h3 | 1.17em | bold | 0.83em 0 | — | Heading level 3 |
| h4 | 1em | bold | 1em 0 | — | Heading level 4 |
| h5 | 0.83em | bold | 1.17em 0 | — | Heading level 5 |
| h6 | 0.67em | bold | 1.33em 0 | — | Heading level 6 |
| p | — | — | 1em 0 | — | Paragraph |
| ul | — | — | 1em 0 | 40px left | Unordered list |
| ol | — | — | 1em 0 | 40px left | Ordered list |
| li | — | — | 0 | — | List item |
| dl | — | — | 1em 0 | — | Definition list |
| dt | — | bold | 0.5em top | — | Definition term |
| dd | — | — | 1.5em left | — | Definition data |
| blockquote | — | — | 1em 0 | 1em left | Block quotation |
| pre | monospace | — | 1em 0 | — | Preformatted text |
| code | monospace | — | — | — | Inline code |
| hr | — | — | 1em 0 | — | Horizontal rule |
| address | — | italic | 1em 0 | — | Contact information |
| article | — | — | 1em 0 | — | Article section |
| aside | — | — | 1em 0 | — | Aside/sidebar |
| section | — | — | 1em 0 | — | Section grouping |
| header | — | — | 1em 0 | — | Header section |
| footer | — | — | 1em 0 | — | Footer section |
| nav | — | — | 1em 0 | — | Navigation section |
| main | — | — | 1em 0 | — | Main content |
| figure | — | — | 1em 0 (40px L/R) | — | Figure with caption |
| figcaption | — | italic | 0.5em 0 | — | Figure caption |
| form | — | — | 1em 0 | — | Form container |
| fieldset | — | — | 1em 0 | 1em | Field grouping |
| legend | — | — | — | 0 0.5em | Field legend |
| table | — | — | 1em 0 | — | Data table |
| a | — | — | — | — | Color: #0000ff, underline |
| b, strong | — | bold | — | — | Strong emphasis |
| i, em | — | italic | — | — | Emphasis |
| u | — | — | — | — | Underline decoration |
| s, del | — | — | — | — | Strikethrough decoration |
| body | — | — | 8px | — | Document body |

### Implementation Notes

- **Margin values in em**: Relative to element's computed font size
- **List indentation**: Lists use 40px left padding (standard browser default)
- **Semantic elements**: HTML5 elements (article, section, etc.) have default 1em margin top/bottom
- **Form elements**: Fieldset has 1em margin and padding with border
- **Viewport-aware**: All margin values are computed relative to viewport

### References

- [HTML Standard Rendering (Appendix B)](https://html.spec.whatwg.org/multipage/rendering.html)
- [CSS Cascading and Inheritance](https://www.w3.org/TR/css-cascade-4/#cascade-origin)

---

## 7. Debug Logging

Enable logging via `DEBUG_HTML_PARSER` environment variable or const flag:

```rust
const DEBUG_TOKENIZER: bool = false;
const DEBUG_TREE_BUILDER: bool = false;
const DEBUG_DOM: bool = false;
```

Log levels:
- Token emission
- Insertion mode transitions
- DOM mutations (create element, create text, append child)

---

## References

- [WHATWG HTML Living Standard](https://html.spec.whatwg.org/multipage/)
- [HTML Parsing Spec](https://html.spec.whatwg.org/multipage/parsing.html)
- [DOM Standard](https://dom.spec.whatwg.org/)
