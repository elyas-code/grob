# Style Module

## Overview

The Style module handles CSS processing and the application of styles to DOM elements. It implements CSS cascade resolution, inheritance, and computes final style values for layout and rendering.

## Purpose

- Parse and organize CSS rules from stylesheets
- Match CSS selectors to DOM elements
- Calculate computed style values using CSS cascade rules
- Handle CSS inheritance and property defaults
- Provide style information to layout and paint stages

## Key Concepts

### CSS Cascade

The style module implements the CSS cascade algorithm:

1. **User Agent Styles**: Default browser styles
2. **Author Styles**: Styles from website stylesheets
3. **User Styles**: User-defined styles
4. **Specificity**: Higher specificity rules override lower ones
5. **Importance**: `!important` declarations have highest priority
6. **Order**: Later rules override earlier ones

### Specificity

CSS specificity is calculated based on:
- ID selectors: Highest weight
- Class selectors and attributes: Medium weight
- Element selectors: Lowest weight

### Inheritance

Certain CSS properties automatically inherit from parent elements:
- Text properties: color, font-size, font-family
- Layout properties: Some margin/padding rules
- Visibility: opacity, visibility

## Data Structures

### Style Object

Stores computed CSS properties for an element:
- Property values (color, size, position, etc.)
- Computed dimensions and margins
- Font and text properties
- Display and visibility properties

### Stylesheet

Collection of CSS rules with:
- Selectors identifying target elements
- Property declarations
- Rule specificity information

### Computed Values

Values resolved after cascade and inheritance:
- Absolute values (pixels instead of percentages)
- Color specifications (RGB values)
- Font resolution (actual font name and size)

## Processing Pipeline

1. **Collection**: Gather all applicable CSS rules
2. **Matching**: Find rules that match the element
3. **Cascade**: Sort rules by specificity and order
4. **Inheritance**: Apply inherited properties from parent
5. **Defaults**: Add default values for unset properties
6. **Computation**: Calculate final values (resolve relative units)

## Selector Support

- **Element selectors**: `p`, `div`
- **Class selectors**: `.classname`
- **ID selectors**: `#idname`
- **Attribute selectors**: `[href]`, `[type="text"]`
- **Descendant selectors**: `div p`
- **Child selectors**: `div > p`

## Common CSS Properties

The module handles various CSS properties:

**Display & Layout**:
- `display`: block, inline, inline-block, flex, grid, none
- `position`: static, relative, absolute, fixed
- `width`, `height`: Sizing properties
- `margin`, `padding`: Spacing properties

**Text & Font**:
- `color`: Text color
- `font-family`: Typeface
- `font-size`: Text size
- `font-weight`: Text boldness
- `text-align`: Text alignment

**Appearance**:
- `background-color`: Background color
- `border`: Border styling
- `opacity`: Transparency

## Integration with Other Modules

### DOM Module
Walks the DOM tree to apply styles to each element

### Layout Module
Uses computed styles to determine layout algorithms and calculations

### Paint Module
Uses style information to render elements with correct appearance

### JavaScript Module
Provides style information through the style API

## Performance Optimizations

- Caching of computed styles
- Efficient selector matching
- Lazy evaluation of inherited properties
- Quick lookup of style rules

## Standards Compliance

- Implements CSS cascade as per W3C specifications
- Supports CSS2.1 property set
- CSS3 feature support with extensions
- Proper specificity calculation

## Limitations and Future Work

- Limited pseudo-class support (`:hover`, `:active`)
- No media queries support yet
- No CSS custom properties (variables)
- Animation and transition support pending
- Advanced selectors (`:nth-child`, `:not()`) to be added
