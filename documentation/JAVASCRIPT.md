# JavaScript Module

## Overview

The JavaScript module integrates the **Boa JavaScript engine** into the browser, enabling dynamic script execution within web pages. It provides a runtime environment with basic DOM APIs for JavaScript code to interact with the document and handles event execution.

## Purpose

- Execute JavaScript code from HTML documents (inline `<script>` tags)
- Provide JavaScript runtime context with global objects
- Support DOM manipulation capabilities
- Handle JavaScript events and callbacks
- Manage script execution contexts
- Support dynamic content modification

## Architecture

### JavaScript Engine Integration

The module uses **Boa Engine v0.19** - a JavaScript engine written in Rust:

```toml
# Cargo.toml dependencies
boa_engine = "0.19"
```

Features:
- **Parsing**: Parse JavaScript source code using Boa's parser
- **Execution**: Execute parsed code in a Boa Context
- **Garbage Collection**: Automatic via Boa's GC system
- **Native Integration**: Easy Rust-JavaScript interop

### JsRuntime Structure

The core `JsRuntime` struct manages:

```rust
pub struct JsRuntime {
    context: Context,           // Boa execution context
    dom: Rc<RefCell<Option<Dom>>>,  // Reference to DOM tree
}
```

**Key Methods:**

- `JsRuntime::new()` - Create runtime with default globals
- `execute_script(code)` - Execute JavaScript code
- `set_dom(dom)` - Set DOM reference for scripts to access
- `take_dom()` - Retrieve DOM after modifications
- `context_mut()` - Get mutable access to Boa context for advanced operations

### Execution Flow

1. **Initialization**: Parser creates `JsRuntime` before processing HTML
2. **DOM Setup**: `set_dom()` called with parsed DOM tree
3. **Script Detection**: Parser identifies `<script>` elements
4. **Script Execution**: `execute_script()` runs code in order
5. **DOM Updates**: Scripts can modify the DOM through bindings
6. **Rendering**: Layout and paint systems use updated DOM

## Global Objects

### Implemented

#### `console`

```javascript
console.log(...args)       // Print to stdout/logs
console.error(...args)     // Error logging
console.warn(...args)      // Warning logging
console.info(...args)      // Info logging
```

#### `document`

```javascript
document.title                          // Document title
document.createElement(tag)             // Create element (stub)
document.createTextNode(text)           // Create text node (stub)
document.querySelector(selector)        // Query element (stub)
document.querySelectorAll(selector)     // Query elements (stub)
```

#### `window`

```javascript
window                   // Reference to global object
window.setTimeout(fn, ms)       // Timer (no-op stub)
window.setInterval(fn, ms)      // Interval (no-op stub)
window.clearTimeout(id)         // Clear timer (no-op stub)
window.clearInterval(id)        // Clear interval (no-op stub)
```

### Future Implementation

To extend DOM APIs with real functionality:

1. Create native Rust functions using Boa's function registration
2. Implement DOM query/manipulation that references the DOM tree
3. Support event handler binding through JavaScript callbacks
4. Add network APIs (fetch, XMLHttpRequest) when needed

## Script Execution

### Inline Scripts

```html
<script>
    console.log("This executes in the JS runtime");
    document.title = "Updated Title";
</script>
```

### Script Order

Scripts execute in document order before rendering:

1. HTML is parsed, creating DOM nodes
2. When `<script>` tag encountered, content is extracted
3. Script executes in order with previous scripts' globals preserved
4. After all scripts, layout and paint systems process DOM

### Error Handling

```rust
match runtime.execute_script(code) {
    Ok(result) => { /* Handle success */ },
    Err(err) => { /* Handle JS error */ },
}
```

## Current Limitations and Future Work

### Limitations

- ✋ External scripts (`<script src="...">`) not yet implemented
- ✋ DOM APIs are stubs (don't actually modify DOM yet)
- ✋ Event handlers (onclick, etc.) need proper DOM integration
- ✋ No async/await or Promises yet
- ✋ No module system (import/export)
- ✋ No network APIs (fetch, XMLHttpRequest)

### Planned Enhancements

1. **Real DOM Integration**
   - Implement `document.querySelector()` to return actual elements
   - Implement `element.textContent` for content access
   - Support DOM mutations from JavaScript

2. **Event Handlers**
   - Register onclick, onload, etc. handlers with DOM elements
   - Execute handlers when events fire
   - Support event objects and preventDefault()

3. **External Scripts**
   - Parse `<script src="...">` tags
   - Fetch and execute external JavaScript files
   - Handle CORS and script security

4. **Advanced Features**
   - XMLHttpRequest / Fetch API for networking
   - setTimeout/setInterval with actual timer support
   - Local Storage / Session Storage
   - Canvas and WebGL APIs

## Integration Example

```rust
// In the browser or parser:
use grob_engine::js::JsRuntime;
use grob_engine::dom::Dom;

// Create runtime
let mut runtime = JsRuntime::new();

// Parse HTML to create DOM
let dom = parse_html("<html><body>Test</body></html>");

// Bind DOM to runtime
runtime.set_dom(dom);

// Execute inline script
let code = r#"
    console.log("Hello from Boa!");
    document.title = "New Title";
"#;
runtime.execute_script(code)?;

// Retrieve updated DOM
let updated_dom = runtime.take_dom();
```

## Technical Notes

- Boa uses `JsValue` for all JavaScript values
- The `Source` type handles code input (bytes or files)
- `Context` is the execution environment (similar to V8 Isolate)
- All script execution is synchronous for now
- Memory is managed by Boa's garbage collector

## References

- [Boa Engine Documentation](https://docs.rs/boa_engine/)
- [WHATWG Script Processing Model](https://html.spec.whatwg.org/#scriptContentType)
- [ECMAScript Standard (ECMA-262)](https://tc39.es/ecma262/)

- `element.classList.add()`: Add CSS class
- `element.classList.remove()`: Remove CSS class
- `element.setAttribute()`: Set attributes

### Element Creation

- `document.createElement(tag)`: Create new element
- `element.appendChild(child)`: Add child element
- `element.removeChild(child)`: Remove child element
- `element.insertBefore(new, ref)`: Insert before reference

### Element Properties

- `element.id`: Element ID attribute
- `element.className`: CSS classes
- `element.getAttribute()`: Get attribute value
- `element.style`: Inline styles object

## Event System

### Event Types

- **Click**: Mouse click on element
- **Change**: Form field value changed
- **Submit**: Form submission
- **Load**: Resource loading complete
- **DOMContentLoaded**: Document parsing complete

### Event Handling

- `element.addEventListener()`: Register event listener
- `element.removeEventListener()`: Unregister listener
- `element.onclick`: Direct event handler property
- Event bubbling and capturing

### Event Object

Properties available in event handlers:

- `event.type`: Event type name
- `event.target`: Element that triggered event
- `event.preventDefault()`: Cancel default action
- `event.stopPropagation()`: Stop event bubbling

## Built-in Objects

### Window Object

Global browser object:
- `window.document`: Document object
- `window.location`: URL information
- `window.history`: Navigation history
- `window.setTimeout()`: Delayed execution
- `window.setInterval()`: Repeated execution

### Document Object

Root document object:
- Provides DOM access methods
- Contains page metadata
- Manages document loading state

### Console Object

Debugging output:
- `console.log()`: Output messages
- `console.error()`: Output errors
- `console.warn()`: Output warnings

## Standard Library Support

JavaScript built-in functions and objects:

- **Math**: Mathematical operations
- **String**: Text manipulation
- **Array**: Array methods and operations
- **Object**: Object property management
- **JSON**: JSON parsing and stringification
- **RegExp**: Regular expressions

## Script Execution

### Script Tags

Scripts in HTML documents:

```html
<!-- Inline script -->
<script>
  console.log("Page loaded");
</script>

<!-- External script -->
<script src="script.js"></script>
```

### Execution Timing

- **Synchronous**: Script blocks page parsing
- **Deferred**: Execute after page parsing
- **Async**: Execute as soon as available

## Performance Considerations

- **JIT Compilation**: Just-in-time code optimization
- **Garbage Collection**: Memory management
- **Script Caching**: Store compiled scripts
- **Lazy Loading**: Load scripts on demand

## Error Handling

### Error Types

- **SyntaxError**: Invalid script syntax
- **ReferenceError**: Undefined variable
- **TypeError**: Type mismatch
- **RangeError**: Value out of range

### Error Handling

- Try/catch blocks in scripts
- Error event listeners
- Error callbacks in async operations

## Integration with Other Modules

### DOM Module
Provides document structure for JavaScript access

### Network Module
Fetches script files and handles AJAX requests

### Browser Component
Executes scripts during page loading and interaction

### Style Module
JavaScript can modify styles and classes

## Standards Compliance

- **ECMAScript 5**: Core standard support
- **ECMAScript 6**: Modern features (partial)
- **DOM Level 2**: Event handling
- **HTML Living Standard**: APIs

## Supported Features

- Basic script execution
- DOM element selection and manipulation
- Event listeners and handling
- Console logging
- setTimeout/setInterval
- Standard library (Math, String, Array, Object)
- JSON parsing and stringification
- Regular expressions

## Limitations and Planned Features

**Current Limitations**:
- No engine integrated yet
- Limited DOM APIs
- No async/await
- No Promises support
- No fetch API
- No XMLHttpRequest
- No module system (import/export)
- No Web Workers

**Planned Features**:
- Full JavaScript engine integration
- Complete DOM API implementation
- Fetch API for network requests
- XMLHttpRequest support
- Promises and async/await
- Web APIs (setTimeout, requestAnimationFrame)
- Module system support
- Web Workers for background tasks
- Service Workers support

## Security Considerations

**Current Status**:
- No sandboxing yet
- Limited input validation

**Future Plans**:
- Sandboxed script execution
- Content Security Policy
- Same-Origin Policy enforcement
- Input validation and sanitization
- XSS protection

## Example Script

```javascript
// Get element
const button = document.getElementById('myButton');

// Add event listener
button.addEventListener('click', function() {
    // Create new element
    const p = document.createElement('p');
    p.textContent = 'Button clicked!';
    
    // Add to page
    document.body.appendChild(p);
});

// Modify styles
button.style.backgroundColor = 'blue';
button.style.color = 'white';
```

## Debugging

**Current Methods**:
- console.log() output
- Debug logs to file

**Future Methods**:
- JavaScript debugger protocol
- Breakpoints and stepping
- Variable inspection
- Stack trace display

## Performance Optimization

- Minimize script execution time
- Avoid blocking operations
- Use requestAnimationFrame for animations
- Cache DOM references
- Batch DOM modifications

## Memory Management

- Automatic garbage collection
- Circular reference handling
- Memory leak prevention
- Cache cleanup for long-running pages
