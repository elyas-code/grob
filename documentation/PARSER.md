# Parser Module

## Overview

The Parser module is responsible for converting HTML and CSS text into structured data that the browser engine can process. It implements standard parsing algorithms to build the Document Object Model (DOM) and Stylesheet structures.

## Components

### HTML Parser

**Location**: `engine/src/parser/html/`

**Purpose**: 
- Parse HTML documents
- Construct a DOM tree representing the document structure
- Handle malformed HTML gracefully according to HTML5 specifications

**Key Features**:
- Tokenization: Converts HTML text into tokens
- Tree Construction: Builds hierarchical DOM tree
- Error Recovery: Handles invalid HTML markup
- Attribute Parsing: Extracts element attributes

**Output**: `Dom` structure with nested nodes representing the document

### CSS Parser

**Location**: `engine/src/parser/css/`

**Purpose**:
- Parse CSS stylesheets
- Extract CSS rules, selectors, and properties
- Build stylesheet representation

**Key Features**:
- Rule Parsing: Identifies CSS rules and their selectors
- Property Parsing: Extracts CSS declarations
- Value Parsing: Parses various CSS value types
- Selector Support: Supports element, class, ID, and attribute selectors

**Output**: `Stylesheet` structure containing CSS rules

## Parsing Stages

### HTML Parsing Pipeline

1. **Input**: Raw HTML text
2. **Tokenization**: Break text into meaningful tokens (tags, text, attributes)
3. **Tree Construction**: Build DOM tree by processing tokens
4. **Error Handling**: Recover from malformed markup
5. **Output**: Complete DOM tree

### CSS Parsing Pipeline

1. **Input**: Raw CSS text
2. **Tokenization**: Identify CSS tokens (selectors, properties, values)
3. **Rule Parsing**: Group tokens into CSS rules
4. **Value Parsing**: Parse CSS value expressions
5. **Output**: Stylesheet with rules ready for application

## Data Structures

### DOM Node Types

- Element: HTML elements (div, p, a, etc.)
- Text: Text content within elements
- Comment: HTML comments
- Document: Root of the document tree

### CSS Rule Structure

- Selectors: Identifies which elements the rule applies to
- Properties: CSS declarations (property: value pairs)
- Specificity: Calculated for cascade resolution

## Standards Compliance

- **HTML5**: Follows HTML5 parsing algorithm specification
- **CSS Level 3**: Supports CSS3 selectors and properties
- **Error Handling**: Implements HTML5 error recovery mechanisms

## Limitations and Future Work

- Limited CSS feature support (some advanced selectors, media queries)
- No support for JavaScript within HTML
- Limited entity handling
- Pseudo-element and pseudo-class support to be expanded

## Usage

The parser is typically called from the browser when loading a new document:

```rust
// Parse HTML
let html_parser = HtmlParser::new(html_source);
let dom = html_parser.parse();

// Parse CSS
let css_parser = CssParser::new(css_source);
let stylesheet = css_parser.parse();
```

## Performance Considerations

- Streaming parsing for large documents
- Single-pass parsing algorithm
- Minimal memory allocations during tokenization
- Efficient tree construction algorithms
