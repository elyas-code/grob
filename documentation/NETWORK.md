# Network Module

## Overview

The Network module handles all network communication for the browser. It manages HTTP requests, responses, resource fetching, and provides caching mechanisms for efficient resource loading.

## Purpose

- Handle HTTP requests and responses
- Fetch web resources (HTML, CSS, images, fonts)
- Manage network timeouts and retries
- Implement resource caching
- Handle redirects and authentication
- Support various content types

## Key Concepts

### Network Manager

Central component coordinating network operations:
- Manages active connections
- Implements request queuing
- Handles resource caching
- Manages connection pooling
- Tracks download progress

### Resource Types

Different types of resources that can be fetched:

| Type | Purpose | Example |
|------|---------|---------|
| HTML | Document content | index.html |
| CSS | Stylesheets | style.css |
| Image | Visual content | image.png |
| Font | Typography | roboto.ttf |
| Script | Executable code | script.js |

## HTTP Protocol

The module implements HTTP/1.1 protocol:

- **Request**: Method (GET, POST), URL, headers
- **Response**: Status code, headers, body
- **Methods**: GET for fetching, POST for forms
- **Status Codes**: Success (2xx), Redirect (3xx), Error (4xx, 5xx)

## Request Types

### GET Requests

Primary method for fetching resources:
- Retrieves resource content
- No request body
- Used for HTML, CSS, images, fonts
- Can be cached

### POST Requests (Planned)

For form submissions and data:
- Includes request body
- Used for form data submission
- Generally not cached
- Requires more bandwidth

## Response Handling

### Status Codes

- **200 OK**: Successful request
- **301/302 Redirect**: Resource moved
- **304 Not Modified**: Cached content valid
- **404 Not Found**: Resource doesn't exist
- **500 Error**: Server error

### Content Types

Handled content MIME types:
- `text/html`: HTML documents
- `text/css`: CSS stylesheets
- `image/*`: Various image formats
- `font/*`: Font files
- `text/javascript`: JavaScript code

## Caching Strategy

Resources are cached to improve performance:

### Cache Types

- **Memory Cache**: Fast in-memory storage
- **Disk Cache**: Persistent storage (planned)
- **Browser Cache**: Managed by HTTP cache headers

### Cache Control

Determined by HTTP headers:
- `Cache-Control`: Caching directives
- `Expires`: Expiration date
- `ETag`: Entity tag for validation
- `Last-Modified`: Last modification time

### Cache Validation

Checking if cached content is still valid:
- **If-Modified-Since**: Conditional request
- **ETag**: Strong validation
- **Expires**: Time-based validation

## Request Pipeline

```
URL Input
    ↓
Check Cache
    ↓
If cached and valid → Return cached
If not valid → Send request
    ↓
HTTP Request
    ↓
Response Received
    ↓
Parse Response Headers
    ↓
Cache if applicable
    ↓
Return Content
```

## Timeouts and Retries

Network resilience features:

- **Connection Timeout**: Maximum time to connect
- **Read Timeout**: Maximum time to receive data
- **Retries**: Automatic retry on failure
- **Backoff**: Exponential backoff for retries

## Redirect Handling

Automatic redirect support:

- **3xx Status Codes**: Indicate redirect
- **Location Header**: New URL destination
- **Chain Following**: Up to limit (default 5)
- **Infinite Loop Detection**: Prevent redirect loops

## Headers Management

HTTP header handling:

### Request Headers

- `User-Agent`: Browser identification
- `Accept`: Accepted content types
- `Accept-Encoding`: Compression support
- `Referer`: Referring page
- `Cookie`: Session and tracking data

### Response Headers

- `Content-Type`: MIME type of response
- `Content-Length`: Size of response body
- `Content-Encoding`: Compression method
- `Set-Cookie`: Session cookies

## Integration with Other Modules

### Parser Module
Provides HTML and CSS content for parsing

### Font Module
Fetches font files from URLs

### Browser Component
Initiates resource requests for page loading

### JavaScript Module
Handles XMLHttpRequest and fetch API

## Performance Features

- **Connection Pooling**: Reuse connections
- **Pipelining**: Multiple requests per connection
- **Compression**: GZIP/DEFLATE support
- **Parallel Downloads**: Multiple simultaneous requests
- **Priority Queuing**: Prioritize critical resources

## Standards Compliance

- HTTP/1.1 specification (RFC 7230-7235)
- URL specification
- MIME types standard
- Cache control specifications
- Cookie specifications (RFC 6265)

## Supported Features

- HTTP GET requests
- HTTP response parsing
- Status code handling
- Basic header support
- Memory caching
- Simple redirect handling
- Timeout support

## Limitations and Planned Features

**Current Limitations**:
- HTTP/1.1 only (no HTTP/2)
- No disk cache persistence
- Limited POST support
- No authentication
- No compression (GZIP/DEFLATE)
- No proxy support
- No SSL/TLS (HTTP only)

**Planned Features**:
- HTTP/2 support
- Persistent disk cache
- Full POST support
- Basic authentication
- Compression support
- HTTPS/SSL support
- Proxy support
- Cookie management
- Form data handling

## Security Considerations

**Current Status**:
- No HTTPS support yet
- Limited header validation
- No CSRF protection

**Future Plans**:
- HTTPS/TLS support
- Security header validation
- CORS support
- CSRF token handling
- Content Security Policy

## Error Handling

Network errors are handled gracefully:

- **Connection Errors**: Network unreachable
- **Timeout Errors**: Request took too long
- **Protocol Errors**: Invalid response format
- **HTTP Errors**: 4xx and 5xx status codes
- **DNS Errors**: Domain name resolution failure

## Resource Types Configuration

Different resources may have different settings:

```rust
// HTML document - higher priority
request.priority = RequestPriority::High;
request.timeout = Duration::from_secs(30);

// Image - can be lower priority
request.priority = RequestPriority::Low;
request.timeout = Duration::from_secs(15);

// Font - medium priority
request.priority = RequestPriority::Medium;
request.timeout = Duration::from_secs(20);
```

## Bandwidth Optimization

Techniques to reduce bandwidth usage:

- **Caching**: Avoid redundant downloads
- **Compression**: GZIP to reduce size
- **Image Optimization**: Scaled images
- **Lazy Loading**: Load resources on demand
- **Resource Prioritization**: Load critical first
