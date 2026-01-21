# Platform Module

## Overview

The Platform module provides cross-platform abstractions that allow the browser engine to run on multiple operating systems. It handles platform-specific functionality while maintaining a unified interface for other modules.

## Purpose

- Abstract platform-specific operations
- Provide OS-independent APIs
- Handle system resource management
- Enable code portability
- Manage platform capabilities
- Support multiple operating systems

## Key Concepts

### Platform Abstraction Layer

A layer that hides OS-specific details:

```
Grob Engine (Platform-independent)
        ↓
Platform Module (Abstraction)
        ↓
↙         ↓         ↘
Windows   macOS     Linux
```

### Supported Platforms

- **Windows**: Using Win32 and system APIs
- **macOS**: Using Cocoa and system frameworks
- **Linux**: Using X11 and GTK (Planned)

## Core Functionality

### File System Operations

Platform-independent file access:

- **File Reading**: Load files from disk
- **File Writing**: Save data to disk
- **Path Handling**: Normalize paths across platforms
- **Directory Operations**: List, create, delete directories
- **Permissions**: Check file permissions

### System Information

Retrieve system and environment information:

- **OS Type**: Detect Windows, macOS, Linux
- **Architecture**: CPU architecture (x86, ARM)
- **Memory**: Available system memory
- **Display**: Screen resolution and DPI
- **Locale**: System language and region

### Resource Management

Manage system resources efficiently:

- **Memory**: Allocate and free memory
- **Timers**: High-resolution timing
- **Threads**: Multi-threading support (Planned)
- **Locks**: Synchronization primitives

### Input/Output

Handle user input and system output:

- **Keyboard**: Keyboard input processing
- **Mouse**: Mouse movement and clicks
- **Touch**: Touch screen input (Planned)
- **Clipboard**: Copy/paste operations

### Graphics and Display

Graphics-related operations:

- **Screen DPI**: Device pixel ratio
- **Color Space**: RGB, sRGB support
- **Rendering Backend**: Graphics API selection
- **Display Scaling**: Handle different screen scales

## Abstraction Layers

### File I/O Abstraction

```rust
trait FileSystem {
    fn read_file(path: &Path) -> Result<Vec<u8>>;
    fn write_file(path: &Path, data: &[u8]) -> Result<()>;
    fn exists(path: &Path) -> bool;
    fn is_directory(path: &Path) -> bool;
}
```

### System Info Abstraction

```rust
trait SystemInfo {
    fn get_os() -> OperatingSystem;
    fn get_screen_dpi() -> f32;
    fn get_available_memory() -> u64;
    fn get_processor_count() -> usize;
}
```

## Platform-Specific Implementations

### Windows Implementation

- **File System**: Windows API for file operations
- **Paths**: NTFS path handling with drive letters
- **Registry**: Windows registry access (Planned)
- **Fonts**: Windows font directories
- **Graphics**: Direct3D or GDI+ (Planned)

### macOS Implementation

- **File System**: POSIX file operations
- **Paths**: Unix-style paths
- **Frameworks**: Cocoa for native integration
- **Fonts**: macOS font directories
- **Graphics**: Metal or OpenGL (Planned)

### Linux Implementation (Planned)

- **File System**: Linux filesystem operations
- **Paths**: Unix-style paths
- **Desktop**: X11 and GTK integration
- **Fonts**: Linux font paths
- **Graphics**: OpenGL or Vulkan

## Environment Variables

Configuration through environment variables:

- `GROB_DATA_DIR`: Custom data directory
- `GROB_LOG_LEVEL`: Logging verbosity
- `GROB_CACHE_DIR`: Cache directory location
- `GROB_TEMP_DIR`: Temporary file directory

## Configuration Files

Platform-dependent configuration:

- **Windows**: Registry or ini files
- **Unix**: .config/ or .grob/ directories
- **Shared**: Standard locations via XDG

## Font System Integration

Different font handling per platform:

- **Windows**: System32/Fonts directory, registry lookup
- **macOS**: /Library/Fonts, ~/Library/Fonts
- **Linux**: /usr/share/fonts, ~/.fonts

## Network Configuration

Platform-specific network setup:

- **Proxy Settings**: System proxy configuration
- **DNS**: System DNS resolver
- **Certificates**: System certificate store

## Display and DPI Awareness

Handle different screen densities:

- **DPI Scaling**: Pixel ratio calculation
- **High DPI**: Support for Retina and 4K displays
- **Scale Factors**: Convert between logical and physical pixels
- **Multiple Monitors**: Multi-display support (Planned)

## Performance and Optimization

Platform-specific optimizations:

- **CPU Features**: Use SIMD where available
- **Memory**: Page size and caching strategies
- **Threading**: Thread pool configuration
- **Caching**: Platform-specific cache strategies

## Integration with Other Modules

### Browser Component
Uses platform module for window management and input

### Network Module
Platform-specific HTTP implementation

### Font Module
Accesses system font directories

### Paint Module
Platform-specific graphics rendering

## Testing Across Platforms

Cross-platform testing strategies:

- **Platform Detection**: Conditional compilation
- **Fallbacks**: Graceful degradation
- **Mocking**: Mock platform operations for testing
- **CI/CD**: Test on multiple platforms

## Standards and Conventions

### Path Handling

Normalize paths across platforms:

```rust
// Input: "C:\\Users\\Downloads\\file.txt" (Windows)
// Normalized: "c:/users/downloads/file.txt" (Internal)
// Output: Platform-specific when needed
```

### Line Endings

Handle CRLF (Windows) vs LF (Unix):

- **Reading**: Normalize to LF internally
- **Writing**: Convert to platform-specific endings

### Character Encoding

Support multiple character encodings:

- **UTF-8**: Primary encoding (internal)
- **UTF-16**: Windows APIs
- **Local Encoding**: Platform-specific

## Security Considerations

**Current Status**:
- Basic path validation
- File access permissions check

**Future Plans**:
- Sandboxing
- Security context isolation
- Safe file operations
- Permission management

## Future Enhancements

- Linux support
- Wayland support (alternative to X11)
- Touch input support
- Gamepad/controller support
- Microphone/camera access
- Platform notifications
- System tray integration
- Drag and drop support
- Multiple window support

## Dependencies

- Platform-specific system libraries
- Graphics APIs (Direct3D, Metal, Vulkan)
- Font loading libraries
- Network libraries

## Compatibility

**Tested Platforms**:
- Windows 10/11
- Ubuntu 20.04+
- macOS 10.15+ (Planned)

**Minimum Requirements**:
- 64-bit processor
- 2GB RAM
- OpenGL 3.0 or equivalent
