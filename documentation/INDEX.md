# Grob Browser - Documentation Hub

Welcome to the Grob Browser documentation. This is your central hub for understanding the architecture, components, and operation of the Grob web browser engine.

## Quick Overview

**Grob** is a modular, high-performance web browser engine written in Rust. It's designed from the ground up to be fast, safe, and easy to understand. The architecture separates concerns into specialized modules that work together to transform HTML and CSS into rendered web pages.

**Current Status**: Active Development (Alpha)

**Key Features**:
- Written in Rust for memory safety and performance
- Modular architecture with clear separation of concerns
- Cross-platform support (Windows, macOS, Linux planned)
- Standards-compliant HTML/CSS processing
- Extensible design for future enhancements

---

## Documentation Index

### Core Rendering Pipeline
The heart of the browser - how content transforms into pixels:

1. **[Parser](PARSER.md)** - HTML and CSS parsing
   - Converts text into structured data
   - Handles malformed input gracefully
   - Generates DOM and Stylesheet objects

2. **[DOM](DOM.md)** - Document Object Model
   - In-memory representation of page structure
   - Element hierarchy and tree navigation
   - Foundation for styling and layout

3. **[Style](STYLE.md)** - CSS Processing
   - CSS cascade and specificity rules
   - Style matching and inheritance
   - Computed values for layout

4. **[Layout](LAYOUT.md)** - Box Model & Positioning
   - CSS box model implementation
   - Element dimension and position calculation
   - Support for block, inline, and flex layouts

5. **[Paint](PAINT.md)** - Rendering Engine
   - Converts layout to visual output
   - Drawing elements, text, and decorations
   - Stacking context and z-index handling

### Support Systems
Specialized modules that enable core functionality:

6. **[Font](FONT.md)** - Typography System
   - Font loading and caching
   - Font metrics and glyph rendering
   - System and web font support

7. **[Network](NETWORK.md)** - HTTP & Resources
   - HTTP request/response handling
   - Resource fetching and caching
   - URL loading and redirect support

8. **[JavaScript](JAVASCRIPT.md)** - Scripting Engine
   - JavaScript execution integration
   - DOM API exposure to scripts
   - Event handling and callbacks

9. **[Platform](PLATFORM.md)** - OS Abstraction
   - Cross-platform compatibility layer
   - File system and system information access
   - Platform-specific implementations

### Application & Architecture

10. **[Browser](BROWSER.md)** - Application Layer
    - Window management and user interface
    - Event loop and rendering coordination
    - User interaction handling

11. **[Architecture](ARCHITECTURE.md)** - System Design
    - High-level component relationships
    - Data flow and processing pipeline
    - Module dependencies and interfaces

---

## Getting Started

### Understanding the System

**New to Grob?** Start here:
1. Read this index overview
2. Review [Architecture](ARCHITECTURE.md) for the big picture
3. Pick a component that interests you based on your needs

**Want to know...**
- **How HTML becomes a page?** → See the Core Pipeline section above
- **How styles are applied?** → Read [Style](STYLE.md)
- **How text is rendered?** → Check [Font](FONT.md)
- **How pages load?** → Look at [Network](NETWORK.md)

### Building & Running

```bash
# Build the project
cargo build

# Run in debug mode
cargo build --debug
cargo run -p grob_browser

# Run in release mode (optimized)
cargo build --release
cargo run --release -p grob_browser
```

See the main README.md for detailed build instructions.

---

## Architecture at a Glance

### Rendering Pipeline

```
HTML Input
    ↓
[Network] → Fetch resources
    ↓
[Parser] → Build DOM & Stylesheets
    ↓
[Style] → Apply CSS rules
    ↓
[Layout] → Calculate positions/sizes
    ↓
[Paint] → Generate rendering instructions
    ↓
[Browser] → Display in window
```

### Component Dependencies

```
Browser (Application)
  ↓
Engine (Core)
  ├─ Parser ─────────────────┐
  ├─ DOM                     │
  ├─ Style ─────────────────┐│
  ├─ Layout ────────────────┐││
  ├─ Paint ─────────────────┐│││
  ├─ Font ──────────────────┐│││
  ├─ Network ────────────────┐│││
  ├─ JavaScript ────────────┐│││
  └─ Platform ──────────────┐│││
```

---

## Component Overview

| Component | Purpose | Status | Key Features |
|-----------|---------|--------|--------------|
| Parser | HTML/CSS parsing | ✅ Core | Tree building, error recovery |
| DOM | Document structure | ✅ Core | Tree navigation, attributes |
| Style | CSS processing | ✅ Core | Cascade, inheritance, specificity |
| Layout | Positioning & sizing | ✅ Core | Box model, block/inline layout |
| Paint | Rendering | ✅ Core | Drawing, text, stacking contexts |
| Font | Typography | ✅ Working | Font loading, metrics, glyphs |
| Network | HTTP requests | ✅ Working | Caching, timeouts, redirects |
| JavaScript | Scripting | 🚧 Planned | Engine integration, DOM APIs |
| Browser | Application UI | ✅ Working | Window, events, rendering loop |
| Platform | OS abstraction | ✅ Working | Windows support, macOS/Linux planned |

---

## Frequently Asked Questions

### General Questions

**Q: What is Grob?**
A: Grob is an open-source web browser engine written in Rust, focusing on modularity, safety, and performance. See [Architecture](ARCHITECTURE.md).

**Q: How does it work?**
A: HTML is parsed into a DOM, CSS styles are applied, layout is calculated, and finally everything is painted to pixels. See the Core Pipeline above.

**Q: Why Rust?**
A: Rust provides memory safety without garbage collection, preventing entire classes of bugs while maintaining high performance.

### Component-Specific

**Q: Where is HTML parsing handled?**
A: [Parser module](PARSER.md) - specifically the HTML parser sub-module.

**Q: How are fonts loaded?**
A: [Font module](FONT.md) - loads from system directories and web sources, with caching.

**Q: How does styling work?**
A: [Style module](STYLE.md) - implements CSS cascade, specificity, and inheritance rules.

**Q: How are elements positioned?**
A: [Layout module](LAYOUT.md) - calculates positions using the CSS box model.

**Q: How is content drawn?**
A: [Paint module](PAINT.md) - converts layout into rendering instructions, handles stacking.

**Q: How are web pages fetched?**
A: [Network module](NETWORK.md) - handles HTTP requests, caching, and resource loading.

**Q: How does JavaScript work?**
A: [JavaScript module](JAVASCRIPT.md) - integrates a JS engine and provides DOM APIs (in development).

### Development

**Q: How do I add a new CSS property?**
A: Modify the [Style module](STYLE.md) to recognize and handle the property in the cascade and inheritance logic.

**Q: How do I add support for a new HTML element?**
A: The parser already handles HTML generically - style and layout handle elements via attributes/types.

**Q: How do I port to a new platform?**
A: Implement the [Platform module](PLATFORM.md) abstraction for the target OS.

**Q: Where should I look to fix a rendering bug?**
A: Start with [Paint module](PAINT.md) for visual issues, [Layout module](LAYOUT.md) for positioning problems.

---

## Contributing & Development

### Adding Features

**Before starting:**
1. Review [Architecture.md](ARCHITECTURE.md) to understand where your feature belongs
2. Check the relevant component's documentation
3. Understand the module's current limitations

**Common tasks:**
- **New CSS property**: Modify [Style](STYLE.md)
- **New layout mode**: Extend [Layout](LAYOUT.md)
- **New rendering feature**: Update [Paint](PAINT.md)
- **Font handling improvement**: Enhance [Font](FONT.md)
- **Network enhancement**: Extend [Network](NETWORK.md)

### Code Organization

```
grob/
├── browser/              # Browser application
│   └── src/main.rs      # Entry point, event loop
│
├── engine/              # Core rendering engine
│   └── src/
│       ├── parser/      # HTML and CSS parsing
│       ├── dom/         # Document structure
│       ├── style/       # CSS processing
│       ├── layout/      # Positioning engine
│       ├── paint/       # Rendering
│       ├── font/        # Typography
│       ├── network/     # HTTP requests
│       ├── js/          # JavaScript integration
│       └── platform/    # OS abstraction
│
└── documentation/       # This documentation
```

### Testing

- Unit tests in individual modules
- Integration tests for component interactions
- Property-based tests for algorithms

---

## Roadmap & Future Work

### Short Term
- [ ] Complete JavaScript engine integration
- [ ] Add Flexbox layout support
- [ ] Implement Grid layout
- [ ] Add more CSS selector support
- [ ] Improve error messages

### Medium Term
- [ ] Grid layout system
- [ ] CSS animations and transitions
- [ ] Improved image support
- [ ] Better performance optimizations
- [ ] HTTPS/TLS support
- [ ] macOS port

### Long Term
- [ ] WebAssembly support
- [ ] Service Workers
- [ ] Hardware acceleration (GPU rendering)
- [ ] Multi-tab support
- [ ] Developer tools
- [ ] Plugin system

### Component Roadmaps

See individual component docs for specific roadmaps:
- [Parser roadmap](PARSER.md#limitations-and-future-work)
- [Layout roadmap](LAYOUT.md#limitations-and-current-features)
- [Paint roadmap](PAINT.md#limitations-and-future-work)
- [Network roadmap](NETWORK.md#limitations-and-planned-features)
- [JavaScript roadmap](JAVASCRIPT.md#limitations-and-planned-features)

---

## Module Documentation Quick Links

| Module | Purpose | Key Classes | See Also |
|--------|---------|-------------|----------|
| **Parser** | Parse HTML/CSS | HtmlParser, CssParser | [Full Docs](PARSER.md) |
| **DOM** | Document structure | Dom, Node | [Full Docs](DOM.md) |
| **Style** | CSS processing | Style, Stylesheet | [Full Docs](STYLE.md) |
| **Layout** | Positioning | LayoutBox, LayoutEngine | [Full Docs](LAYOUT.md) |
| **Paint** | Rendering | PaintInstruction, DisplayList | [Full Docs](PAINT.md) |
| **Font** | Typography | FontManager, Font | [Full Docs](FONT.md) |
| **Network** | HTTP | NetworkManager, Request | [Full Docs](NETWORK.md) |
| **JavaScript** | Scripting | JavaScriptEngine | [Full Docs](JAVASCRIPT.md) |
| **Platform** | OS abstraction | FileSystem, SystemInfo | [Full Docs](PLATFORM.md) |
| **Browser** | Application | Browser, EventLoop | [Full Docs](BROWSER.md) |

---

## Getting Help

- Check [Frequently Asked Questions](#frequently-asked-questions) above
- Review the specific component documentation
- See [Architecture.md](ARCHITECTURE.md) for system-level understanding
- Check the main project README.md

---

## Learning Path

### For Users
1. Understand [what Grob does](ARCHITECTURE.md)
2. Learn [how to build and run it](../README.md)
3. Try loading a simple HTML page

### For Developers - Beginner
1. Read [Architecture.md](ARCHITECTURE.md) for overview
2. Pick one component and read its full documentation
3. Explore the corresponding source code
4. Try adding a small test case

### For Developers - Intermediate
1. Understand the complete [rendering pipeline](ARCHITECTURE.md)
2. Study 2-3 interconnected modules
3. Trace code through a real scenario (e.g., "loading a page")
4. Try implementing a simple feature

### For Developers - Advanced
1. Review [Architecture.md](ARCHITECTURE.md) and all component docs
2. Understand performance characteristics and trade-offs
3. Plan architectural improvements
4. Implement significant features or optimizations

---

## Documentation Standards

All documentation in this folder follows these conventions:

- **Markdown format** for easy reading and version control
- **Code examples** where helpful
- **ASCII diagrams** for architecture visualization
- **Consistent structure**: Overview → Concepts → Usage → Integration → Limitations
- **Links between related docs**
- **Practical examples** in component docs

---

**Last Updated**: January 22, 2026

For the latest information, see the individual component documentation files above.
