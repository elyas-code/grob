# Paint Module

## Overview

The Paint module is the final stage in the rendering pipeline. It takes the styled DOM tree and calculated layout information, and converts it into visual output. The module handles drawing elements, text rendering, and compositing.

## Purpose

- Convert layout tree into rendering instructions
- Draw background colors and borders
- Render text content
- Handle z-index and stacking contexts
- Composite layers for final display
- Apply visual effects (opacity, shadows)

## Rendering Pipeline

```
Styled DOM + Layout Tree
        ↓
   Paint Operations
   - Backgrounds
   - Borders
   - Text
   - Images
        ↓
   Stacking Context
   (z-index ordering)
        ↓
   Compositing
        ↓
   Pixel Output
```

## Key Concepts

### Stacking Context

Defines the order in which elements are drawn:
- Elements are painted back-to-front
- `z-index` determines order within context
- Each context is self-contained
- Respects CSS stacking rules

### Painting Order

Standard painting order for each element:

1. **Background**: Background color and images
2. **Borders**: Border styling
3. **Content**: Text and inline content
4. **Children**: All child elements (recursively)
5. **Overlays**: Decorations and effects

## Data Structures

### PaintInstruction

Represents a single painting operation:
- **Type**: Background, border, text, image, etc.
- **Bounds**: Rectangle defining painted area
- **Properties**: Color, size, font, etc.

### DisplayList

Ordered list of paint instructions for complete page:
- Sorted by stacking context and z-index
- Contains all information needed for rendering
- Can be cached and reused

## Rendering Operations

### Background Rendering

- Fill element with background color
- Support for background images (planned)
- Gradient backgrounds (planned)

### Border Rendering

- Draw borders with specified width and style
- Support for different border styles (solid, dashed, etc.)
- Border radius for rounded corners (planned)

### Text Rendering

- Render text glyphs using Font Manager
- Apply text color and styling
- Handle text alignment and overflow
- Support for underline, strikethrough

### Image Rendering (Planned)

- Load and cache images
- Render images at calculated dimensions
- Handle image scaling and alignment

## Integration with Other Modules

### Layout Module
Uses layout box dimensions and positions for rendering boundaries

### Style Module
Reads computed styles (colors, fonts, borders) for rendering

### Font Module
Requests glyphs and metrics for text rendering

### Browser Component
Outputs to pixel buffer for display

## Performance Optimizations

- Caching of paint results
- Dirty region tracking (invalidation)
- Clipping to visible areas
- Batching of similar operations
- Culling of off-screen elements

## Coordinate Systems

### Logical Coordinates

- Used internally for calculations
- Device-independent
- Used by DOM and layout

### Physical Coordinates

- Actual screen pixels
- Device-dependent
- Used for final rendering
- Scaled by DPI factor

## Layering and Compositing

Elements are painted in layers:
- **Opaque layers**: Solid background elements
- **Transparent layers**: Elements with opacity
- **Blending**: Combination of overlapping layers

## Standards Compliance

- Follows CSS Painting Order specification
- Respects CSS stacking context rules
- Implements standard z-index behavior
- CSS2.1 visual effects support

## Supported Features

- Background colors
- Border drawing
- Text rendering
- Text color and decoration
- Element stacking and z-index
- Opacity (alpha transparency)
- Basic clipping

## Limitations and Future Work

- No image rendering yet
- No background images
- No gradients support
- No shadows (box-shadow, text-shadow)
- No filters
- Limited text decoration options
- No rotation or transform effects
- Rounded corners not yet implemented

## Rendering Quality

The paint module prioritizes:
- **Correctness**: Accurate color and position
- **Performance**: Efficient rasterization
- **Simplicity**: Maintainable code
- **Extensibility**: Easy to add new rendering types

## Example Paint Sequence

```rust
// For a styled div with text:
1. Paint background color (blue)
2. Paint borders (2px solid black)
3. Paint text content ("Hello", black color)
4. Paint child elements
```

## Color Representation

Colors are typically represented as:
- RGB: Red, Green, Blue components (0-255)
- Alpha: Transparency (0-255, 255 = opaque)
- Hex: #RRGGBB format

## Future Enhancements

- Full image support
- Gradient backgrounds
- Shadow effects (box-shadow, text-shadow)
- Transform operations
- Filter effects
- CSS animations support
- Hardware acceleration
