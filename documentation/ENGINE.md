# Engine Component

## Overview

The Engine is the core of the Grob browser, implementing the entire web standards processing pipeline. It handles HTML/CSS parsing, DOM tree construction, style calculation, layout, rendering, and integration with JavaScript and network functionality.

## Architecture

The engine is organized as a collection of specialized modules that work together to transform web content into rendered pixels.

### Processing Pipeline

```
HTML Input
    ↓
[Parser: HTML] → DOM Tree
    ↓
[Parser: CSS] → Stylesheets
    ↓
[Style] → Styled DOM with computed styles
    ↓
[Layout] → Layout tree with calculated dimensions
    ↓
[Paint] → Rendering instructions
    ↓
Pixel Output
```

## Core Modules

### parser
**Purpose**: Parse HTML and CSS content

- **HTML Parser**: Constructs a DOM tree from HTML source
- **CSS Parser**: Processes stylesheets and CSS rules
- Implements standard parsing algorithms for web content

### dom
**Purpose**: Document Object Model representation

- Represents the hierarchical structure of web documents
- Stores node information (elements, text, attributes)
- Supports DOM tree navigation and manipulation

### style
**Purpose**: CSS style processing and cascading

- Applies CSS rules to DOM nodes
- Resolves the CSS cascade (specificity, inheritance)
- Computes final style values for layout

### layout
**Purpose**: Calculate element positions and dimensions

- Implements the CSS box model
- Calculates layout for flow, flex, and grid layouts
- Determines positioning of all elements on the page

### paint
**Purpose**: Render styled and laid-out content to pixels

- Converts layout tree to rendering instructions
- Handles drawing of elements, text, and decorations
- Implements z-index and stacking context rules

### font
**Purpose**: Font loading, management, and rendering

- Loads and caches fonts from the system and web sources
- Manages font metrics (ascent, descent, line height)
- Provides glyph rendering for text display

### js
**Purpose**: JavaScript engine integration

- Integrates JavaScript runtime
- Handles script execution within the page context
- Provides DOM APIs for JavaScript to interact with content

### net
**Purpose**: Network and HTTP operations

- Handles HTTP requests for resources (HTML, CSS, images, fonts)
- Manages network timeouts and error handling
- Supports resource caching

### platform
**Purpose**: Platform-specific abstraction layer

- Provides cross-platform abstractions for OS operations
- Handles file system access and resource loading
- Manages platform-specific rendering details

## Usage Flow

1. **Initialization**: Create engine components (DOM, Layout Engine, Font Manager)
2. **Parsing**: Parse HTML to build DOM tree
3. **Styling**: Apply CSS and calculate computed styles
4. **Layout**: Calculate element positions and dimensions
5. **Painting**: Render content to pixel buffer
6. **Interaction**: Handle user input and update DOM

## Dependencies

- Standard Rust libraries
- External crates for parsing, layout calculations, and font rendering

## Key Types

- `Dom`: The document tree structure
- `LayoutBox`: Represents layout information for DOM elements
- `Style`: Contains computed CSS properties
- `Stylesheet`: Collection of CSS rules

## Extensibility

The modular architecture allows for:
- Replacement of individual components
- Addition of new layout algorithms
- Support for new CSS features
- Integration of different JavaScript engines
