# Grob Browser

A web browser engine written in Rust, designed to be fast, efficient, and modular.

## Project Structure

```
grob/
├── browser/          # Browser application
├── engine/           # Core browser engine
│   ├── dom/          # Document Object Model implementation
│   ├── font/         # Font handling and rendering
│   ├── js/           # JavaScript engine integration
│   ├── layout/       # Layout calculation engine
│   ├── net/          # Network and HTTP handling
│   ├── paint/        # Rendering and painting
│   ├── parser/       # HTML and CSS parsing
│   ├── platform/     # Platform-specific implementations
│   └── style/        # CSS style processing
└── target/           # Build artifacts
```

## Building

To build the project, ensure you have Rust installed, then run:

```bash
cargo build
```

For a release build with optimizations:

```bash
cargo build --release
```

## Running

To run the browser:

```bash
cargo run -p grob_browser
```

## Components

- **Browser**: The main application entry point
- **Engine**: The core rendering engine that handles:
  - HTML parsing and DOM construction
  - CSS parsing and style calculation
  - Layout computation
  - Text rendering with font support
  - Network requests and HTTP handling
  - JavaScript integration
  - Platform-specific rendering

## Requirements

- Rust 1.56 or later
- Standard development tools (C compiler, etc.) for native dependencies

## Development

To run tests:

```bash
cargo test
```

To check code:

```bash
cargo check
```
