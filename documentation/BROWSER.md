# Browser Component

## Overview

The Browser component is the main application entry point for the Grob web browser. It provides the user interface and window management, handling rendering, user interaction, and network requests.

## Key Features

- **Window Management**: Uses `winit` for cross-platform window and event handling
- **Pixel Rendering**: Utilizes the `pixels` crate for efficient 2D rendering
- **Event Handling**: Manages keyboard, mouse, and window events
- **Font Rendering**: Integrates with the Font Manager for text rendering
- **Network Integration**: Communicates with the Network Manager for HTTP requests
- **DOM Interaction**: Supports hyperlink navigation and DOM manipulation

## Architecture

### Main Entry Point
Located in `browser/src/main.rs`, the browser initializes:
- The Winit event loop for handling OS events
- The pixel buffer for rendering
- The font manager for text rendering
- The layout and rendering pipeline

### Event Loop
The browser's event loop handles:
- Window events (resize, focus, close)
- Mouse events (clicks, movement for link detection)
- Keyboard input
- Rendering and layout updates

### Rendering Pipeline
1. Parse HTML and CSS
2. Build DOM tree
3. Apply styles
4. Calculate layout
5. Paint to pixel buffer
6. Display in window

## Key Modules

- `winit`: Window and event management
- `pixels`: Low-level graphics rendering
- `rusttype`: Font glyph rendering
- `engine`: Core rendering engine integration

## Dependencies

- winit
- pixels
- rusttype
- grob_engine

## Future Enhancements

- Add tab support for multiple pages
- Implement history navigation (back/forward)
- Add developer tools for debugging
- JavaScript engine integration
- Plugin system support
