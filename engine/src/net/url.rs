// URL resolution utilities for resolving relative URLs against base URLs
//
// This module handles:
// - Relative URL resolution using document URL + <base href>
// - srcset attribute parsing and selection
// - URL normalization

/// Represents a parsed URL with its components
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
    pub scheme: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: Option<String>,
    pub fragment: Option<String>,
}

impl ParsedUrl {
    /// Parse a URL string into its components
    pub fn parse(url: &str) -> Option<Self> {
        let url = url.trim();
        
        // Handle data: URLs specially
        if url.starts_with("data:") {
            return Some(ParsedUrl {
                scheme: "data".to_string(),
                host: String::new(),
                port: None,
                path: url[5..].to_string(),
                query: None,
                fragment: None,
            });
        }
        
        // Extract scheme
        let (scheme, rest) = if let Some(pos) = url.find("://") {
            (url[..pos].to_lowercase(), &url[pos + 3..])
        } else {
            return None; // No scheme found
        };
        
        // Extract fragment
        let (rest, fragment) = if let Some(pos) = rest.find('#') {
            (&rest[..pos], Some(rest[pos + 1..].to_string()))
        } else {
            (rest, None)
        };
        
        // Extract query
        let (rest, query) = if let Some(pos) = rest.find('?') {
            (&rest[..pos], Some(rest[pos + 1..].to_string()))
        } else {
            (rest, None)
        };
        
        // Extract host and path
        let (host_port, path) = if let Some(pos) = rest.find('/') {
            (&rest[..pos], rest[pos..].to_string())
        } else {
            (rest, "/".to_string())
        };
        
        // Extract port from host
        let (host, port) = if let Some(pos) = host_port.rfind(':') {
            let potential_port = &host_port[pos + 1..];
            if potential_port.chars().all(|c| c.is_ascii_digit()) {
                (
                    host_port[..pos].to_lowercase(),
                    potential_port.parse().ok(),
                )
            } else {
                (host_port.to_lowercase(), None)
            }
        } else {
            (host_port.to_lowercase(), None)
        };
        
        Some(ParsedUrl {
            scheme,
            host,
            port,
            path,
            query,
            fragment,
        })
    }
    
    /// Convert back to a URL string
    pub fn to_string(&self) -> String {
        if self.scheme == "data" {
            return format!("data:{}", self.path);
        }
        
        let mut url = format!("{}://{}", self.scheme, self.host);
        
        if let Some(port) = self.port {
            url.push(':');
            url.push_str(&port.to_string());
        }
        
        url.push_str(&self.path);
        
        if let Some(ref query) = self.query {
            url.push('?');
            url.push_str(query);
        }
        
        if let Some(ref fragment) = self.fragment {
            url.push('#');
            url.push_str(fragment);
        }
        
        url
    }
    
    /// Get the directory path (path without the filename)
    pub fn directory(&self) -> String {
        if let Some(pos) = self.path.rfind('/') {
            self.path[..=pos].to_string()
        } else {
            "/".to_string()
        }
    }
}

/// Resolve a potentially relative URL against a base URL
/// 
/// Examples:
/// - resolve("https://example.com/a/b.html", "c.png") -> "https://example.com/a/c.png"
/// - resolve("https://example.com/a/b.html", "/c.png") -> "https://example.com/c.png"
/// - resolve("https://example.com/a/", "../c.png") -> "https://example.com/c.png"
/// - resolve("https://example.com/a/", "//cdn.example.com/c.png") -> "https://cdn.example.com/c.png"
pub fn resolve_url(base_url: &str, relative_url: &str) -> String {
    let relative = relative_url.trim();
    
    // Already absolute URL
    if relative.contains("://") {
        return relative.to_string();
    }
    
    // Data URI - return as-is
    if relative.starts_with("data:") {
        return relative.to_string();
    }
    
    let base = match ParsedUrl::parse(base_url) {
        Some(b) => b,
        None => return relative.to_string(),
    };
    
    // Protocol-relative URL (//example.com/path)
    if relative.starts_with("//") {
        return format!("{}:{}", base.scheme, relative);
    }
    
    // Absolute path (/path/to/resource)
    if relative.starts_with('/') {
        let mut result = base.clone();
        result.path = relative.to_string();
        result.query = None;
        result.fragment = None;
        return result.to_string();
    }
    
    // Relative path
    let base_dir = base.directory();
    let combined = format!("{}{}", base_dir, relative);
    
    // Normalize the path (resolve . and ..)
    let normalized_path = normalize_path(&combined);
    
    let mut result = base.clone();
    result.path = normalized_path;
    result.query = None;
    result.fragment = None;
    result.to_string()
}

/// Normalize a path by resolving . and .. segments
pub fn normalize_path(path: &str) -> String {
    let mut segments: Vec<&str> = Vec::new();
    
    for segment in path.split('/') {
        match segment {
            "" | "." => {
                // Skip empty segments and current directory references
                if segments.is_empty() {
                    segments.push(""); // Keep leading slash
                }
            }
            ".." => {
                // Go up one directory, but not past root
                if segments.len() > 1 {
                    segments.pop();
                }
            }
            _ => {
                segments.push(segment);
            }
        }
    }
    
    if segments.is_empty() {
        "/".to_string()
    } else if segments.len() == 1 && segments[0].is_empty() {
        "/".to_string()
    } else {
        segments.join("/")
    }
}

/// Resolve a URL using document base URL and optional <base href>
pub fn resolve_url_with_base(document_url: &str, base_href: Option<&str>, relative_url: &str) -> String {
    // If there's a <base href>, first resolve it against the document URL
    let effective_base = match base_href {
        Some(href) if !href.is_empty() => resolve_url(document_url, href),
        _ => document_url.to_string(),
    };
    
    // Then resolve the relative URL against the effective base
    resolve_url(&effective_base, relative_url)
}

/// Represents a single srcset entry
#[derive(Debug, Clone, PartialEq)]
pub struct SrcsetEntry {
    pub url: String,
    pub width: Option<u32>,     // e.g., 800w
    pub density: Option<f32>,   // e.g., 2x
}

/// Parse a srcset attribute value
/// 
/// Format: "url1 800w, url2 2x, url3"
pub fn parse_srcset(srcset: &str) -> Vec<SrcsetEntry> {
    let mut entries = Vec::new();
    
    for part in srcset.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        
        let tokens: Vec<&str> = part.split_whitespace().collect();
        if tokens.is_empty() {
            continue;
        }
        
        let url = tokens[0].to_string();
        let mut width = None;
        let mut density = None;
        
        // Parse descriptor (if any)
        if tokens.len() > 1 {
            let descriptor = tokens[1].to_lowercase();
            if descriptor.ends_with('w') {
                width = descriptor.trim_end_matches('w').parse().ok();
            } else if descriptor.ends_with('x') {
                density = descriptor.trim_end_matches('x').parse().ok();
            }
        }
        
        // Default density is 1x if not specified
        if width.is_none() && density.is_none() && tokens.len() == 1 {
            density = Some(1.0);
        }
        
        entries.push(SrcsetEntry { url, width, density });
    }
    
    entries
}

/// Select the best image from a srcset based on viewport width and device pixel ratio
/// 
/// Returns the URL of the best matching image
pub fn select_srcset_image(
    srcset: &[SrcsetEntry],
    fallback_src: Option<&str>,
    viewport_width: u32,
    device_pixel_ratio: f32,
) -> Option<String> {
    if srcset.is_empty() {
        return fallback_src.map(|s| s.to_string());
    }
    
    // First, try to find a match based on width descriptors
    let width_candidates: Vec<_> = srcset.iter()
        .filter(|e| e.width.is_some())
        .collect();
    
    if !width_candidates.is_empty() {
        let target_width = (viewport_width as f32 * device_pixel_ratio) as u32;
        
        // Find the smallest image that's >= target width, or the largest if none are big enough
        let mut best = width_candidates[0];
        for entry in &width_candidates {
            let entry_w = entry.width.unwrap();
            let best_w = best.width.unwrap();
            
            if entry_w >= target_width && (best_w < target_width || entry_w < best_w) {
                best = entry;
            } else if best_w < target_width && entry_w > best_w {
                best = entry;
            }
        }
        
        return Some(best.url.clone());
    }
    
    // Fall back to density descriptors
    let density_candidates: Vec<_> = srcset.iter()
        .filter(|e| e.density.is_some())
        .collect();
    
    if !density_candidates.is_empty() {
        // Find the closest match to the device pixel ratio
        let mut best = density_candidates[0];
        let mut best_diff = (best.density.unwrap() - device_pixel_ratio).abs();
        
        for entry in &density_candidates {
            let diff = (entry.density.unwrap() - device_pixel_ratio).abs();
            if diff < best_diff {
                best = entry;
                best_diff = diff;
            }
        }
        
        return Some(best.url.clone());
    }
    
    // Fall back to the first entry
    srcset.first().map(|e| e.url.clone())
        .or_else(|| fallback_src.map(|s| s.to_string()))
}

/// Extract media type from a Content-Type header value
pub fn parse_content_type(header_value: &str) -> String {
    // Content-Type can be "image/png; charset=utf-8"
    if let Some(pos) = header_value.find(';') {
        header_value[..pos].trim().to_lowercase()
    } else {
        header_value.trim().to_lowercase()
    }
}

/// Check if a URL is a data URI
pub fn is_data_uri(url: &str) -> bool {
    url.trim().to_lowercase().starts_with("data:")
}

/// Parse a data URI and extract the content type and data
pub fn parse_data_uri(uri: &str) -> Option<(String, Vec<u8>)> {
    let uri = uri.trim();
    if !uri.to_lowercase().starts_with("data:") {
        return None;
    }
    
    let data_part = &uri[5..]; // Remove "data:"
    
    let (meta, data) = if let Some(pos) = data_part.find(',') {
        (&data_part[..pos], &data_part[pos + 1..])
    } else {
        return None;
    };
    
    let is_base64 = meta.ends_with(";base64");
    let content_type = if is_base64 {
        meta.trim_end_matches(";base64")
    } else {
        meta
    };
    
    let content_type = if content_type.is_empty() {
        "text/plain"
    } else {
        content_type
    };
    
    let bytes = if is_base64 {
        decode_base64(data)?
    } else {
        // URL-decode the data
        url_decode(data).into_bytes()
    };
    
    Some((content_type.to_string(), bytes))
}

/// Simple base64 decoder
fn decode_base64(input: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    fn char_to_value(c: u8) -> Option<u8> {
        ALPHABET.iter().position(|&x| x == c).map(|p| p as u8)
    }
    
    let input: Vec<u8> = input.bytes()
        .filter(|&c| c != b'\n' && c != b'\r' && c != b' ' && c != b'\t')
        .collect();
    
    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    
    for &c in &input {
        if c == b'=' {
            continue;
        }
        
        let value = char_to_value(c)?;
        buffer = (buffer << 6) | value as u32;
        bits += 6;
        
        if bits >= 8 {
            bits -= 8;
            output.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    
    Some(output)
}

/// Simple URL decoder
fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_url() {
        let url = ParsedUrl::parse("https://example.com:8080/path/to/file.html?query=1#fragment");
        assert!(url.is_some());
        let url = url.unwrap();
        assert_eq!(url.scheme, "https");
        assert_eq!(url.host, "example.com");
        assert_eq!(url.port, Some(8080));
        assert_eq!(url.path, "/path/to/file.html");
        assert_eq!(url.query, Some("query=1".to_string()));
        assert_eq!(url.fragment, Some("fragment".to_string()));
    }
    
    #[test]
    fn test_resolve_relative_url() {
        assert_eq!(
            resolve_url("https://example.com/a/b.html", "c.png"),
            "https://example.com/a/c.png"
        );
        
        assert_eq!(
            resolve_url("https://example.com/a/b.html", "/c.png"),
            "https://example.com/c.png"
        );
        
        assert_eq!(
            resolve_url("https://example.com/a/b/c.html", "../d.png"),
            "https://example.com/a/d.png"
        );
        
        assert_eq!(
            resolve_url("https://example.com/a/", "//cdn.example.com/c.png"),
            "https://cdn.example.com/c.png"
        );
    }
    
    #[test]
    fn test_resolve_absolute_url() {
        assert_eq!(
            resolve_url("https://example.com/a/", "https://other.com/b.png"),
            "https://other.com/b.png"
        );
    }
    
    #[test]
    fn test_resolve_with_base_href() {
        assert_eq!(
            resolve_url_with_base(
                "https://example.com/page.html",
                Some("/assets/"),
                "image.png"
            ),
            "https://example.com/assets/image.png"
        );
    }
    
    #[test]
    fn test_parse_srcset() {
        let srcset = "small.jpg 300w, medium.jpg 600w, large.jpg 1200w";
        let entries = parse_srcset(srcset);
        
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].url, "small.jpg");
        assert_eq!(entries[0].width, Some(300));
        assert_eq!(entries[1].url, "medium.jpg");
        assert_eq!(entries[1].width, Some(600));
        assert_eq!(entries[2].url, "large.jpg");
        assert_eq!(entries[2].width, Some(1200));
    }
    
    #[test]
    fn test_parse_srcset_density() {
        let srcset = "normal.jpg 1x, retina.jpg 2x";
        let entries = parse_srcset(srcset);
        
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].density, Some(1.0));
        assert_eq!(entries[1].density, Some(2.0));
    }
    
    #[test]
    fn test_select_srcset_width() {
        let entries = vec![
            SrcsetEntry { url: "small.jpg".to_string(), width: Some(300), density: None },
            SrcsetEntry { url: "medium.jpg".to_string(), width: Some(600), density: None },
            SrcsetEntry { url: "large.jpg".to_string(), width: Some(1200), density: None },
        ];
        
        // For 400px viewport at 1x, should select 600w (smallest >= 400)
        assert_eq!(
            select_srcset_image(&entries, None, 400, 1.0),
            Some("medium.jpg".to_string())
        );
        
        // For 400px viewport at 2x, should select 1200w (smallest >= 800)
        assert_eq!(
            select_srcset_image(&entries, None, 400, 2.0),
            Some("large.jpg".to_string())
        );
    }
    
    #[test]
    fn test_is_data_uri() {
        assert!(is_data_uri("data:image/png;base64,abc123"));
        assert!(is_data_uri("DATA:text/plain,hello"));
        assert!(!is_data_uri("https://example.com/image.png"));
    }
    
    #[test]
    fn test_parse_content_type() {
        assert_eq!(parse_content_type("image/png"), "image/png");
        assert_eq!(parse_content_type("image/jpeg; charset=utf-8"), "image/jpeg");
        assert_eq!(parse_content_type("  TEXT/HTML  "), "text/html");
    }
}
