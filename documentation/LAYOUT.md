# Layout Module

## Overview

The Layout module is responsible for calculating the positions and dimensions of every element on a web page. It implements the CSS box model and layout algorithms to determine where and how large each element should be rendered.

## Purpose

- Calculate dimensions (width, height) for all elements
- Determine positions (x, y coordinates) for all elements
- Handle margin, padding, and border spacing
- Implement various layout modes (block flow, flexbox, grid)
- Respect CSS positioning rules (static, relative, absolute, fixed)
- Handle element stacking and z-index

## Core Concepts

### CSS Box Model

Every element consists of:

```
┌─────────────────────────────────────┐
│          Margin                     │
│  ┌──────────────────────────────┐   │
│  │      Border                  │   │
│  │  ┌────────────────────────┐  │   │
│  │  │      Padding           │  │   │
│  │  │  ┌──────────────────┐  │  │   │
│  │  │  │    Content       │  │  │   │
│  │  │  └──────────────────┘  │  │   │
│  │  └────────────────────────┘  │   │
│  └──────────────────────────────┘   │
└─────────────────────────────────────┘
```

### Layout Tree

A specialized tree structure built from the DOM:
- Each node represents a layout box for an element
- Contains computed dimensions and position
- Includes margin, border, and padding information
- Organized by stacking context

## Key Data Structures

### LayoutBox

Represents layout information for a single element:
- **Dimensions**: width, height, x, y coordinates
- **Margins**: Top, right, bottom, left
- **Padding**: Top, right, bottom, left
- **Border**: Width and styling
- **Content**: Child layout boxes
- **Display type**: Block, inline, flex, grid, none

### Dimensions

Stores calculated measurements:
- Content width and height
- Position coordinates
- Total space including margins

## Layout Algorithms

### Block Layout

Default layout mode for most elements:
- Elements flow vertically (top to bottom)
- Each element takes full available width
- Height determined by content
- Margins collapse between adjacent elements

### Inline Layout

For text and inline elements:
- Elements flow horizontally (left to right)
- Elements only take necessary width
- Multiple elements on same line if space allows
- Baseline alignment for text

### Flexbox Layout (Planned)

Flexible layout system:
- Main axis and cross axis
- Flex grow and shrink
- Justify content and align items

### Grid Layout (Planned)

Two-dimensional layout system:
- Row and column definitions
- Grid placement
- Alignment control

## Processing Pipeline

1. **Input**: DOM tree with computed styles
2. **Box Generation**: Create layout boxes for visible elements
3. **Dimension Calculation**: Compute widths and heights
4. **Position Calculation**: Determine x, y coordinates
5. **Stacking Context**: Organize z-ordering
6. **Output**: Layout tree with all measurements

## Positioning Modes

| Mode | Behavior |
|------|----------|
| `static` | Normal document flow (default) |
| `relative` | Offset from normal position |
| `absolute` | Positioned relative to ancestor |
| `fixed` | Positioned relative to viewport |

## Integration with Other Modules

### Style Module
Reads computed CSS styles and properties

### Paint Module
Uses layout dimensions to render elements at correct positions

### Browser Component
Receives layout tree for final rendering

## Performance Considerations

- Single-pass layout calculation when possible
- Caching of layout results
- Lazy layout recomputation on changes
- Efficient tree traversal

## Standards Compliance

- Implements CSS Box Model as per W3C specifications
- Follows CSS positioning specification
- Compatible with CSS2.1 layout rules
- CSS3 layout module support planned

## Limitations and Current Features

**Supported**:
- Block and inline layout
- Margin collapsing
- Static positioning
- Relative positioning
- Basic absolute positioning
- Width and height calculation
- Padding and margin handling
- Border dimensions

**Not Yet Implemented**:
- Flexbox layout
- Grid layout
- Float layout
- Multi-column layout
- Overflow and scrolling
- Writing modes
- Text layout optimization

## Example Calculation

```
<div style="width: 100px; padding: 10px; margin: 20px;">
  Content
</div>

Results in:
- Margin: 20px all around
- Total width: 100px (content) + 20px (padding) + border
- Margin space outside element: 20px
```

## Future Enhancements

- Flexbox and Grid support for modern layouts
- Performance optimization for large documents
- Scrolling and overflow handling
- Animation layout recalculation
- Responsive layout features
