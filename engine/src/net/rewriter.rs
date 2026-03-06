// HTML rewriter module for transforming image references
//
// This module provides functionality to:
// - Rewrite image URLs to point to cached local files
// - Inline small images as data URIs
// - Transform srcset attributes
// - Update CSS url() references

use crate::dom::{Dom, NodeId, NodeType, ElementData};
use crate::net::{NetworkManager, FetchedResource};
use crate::net::image::{ImageType, detect_image_type, decode_image};
use std::collections::HashMap;

/// Configuration for the HTML rewriter
#[derive(Clone)]
pub struct RewriterConfig {
    /// Maximum size in bytes for inlining as data URI
    pub max_inline_size: usize,
    /// Whether to inline images as data URIs
    pub inline_images: bool,
    /// Target viewport width for srcset selection
    pub viewport_width: u32,
    /// Device pixel ratio for srcset selection
    pub device_pixel_ratio: f32,
}

impl Default for RewriterConfig {
    fn default() -> Self {
        Self {
            max_inline_size: 32 * 1024, // 32KB
            inline_images: true,
            viewport_width: 1920,
            device_pixel_ratio: 1.0,
        }
    }
}

/// Result of image processing
#[derive(Debug, Clone)]
pub struct ProcessedImage {
    /// Original URL
    pub original_url: String,
    /// Resolved absolute URL
    pub resolved_url: String,
    /// The final URL to use (data URI or resolved URL)
    pub final_url: String,
    /// Whether this image was inlined as data URI
    pub inlined: bool,
    /// Image dimensions if available
    pub width: Option<u32>,
    pub height: Option<u32>,
}

/// HTML rewriter that transforms image references
pub struct HtmlRewriter {
    config: RewriterConfig,
    processed_images: HashMap<String, ProcessedImage>,
}

impl HtmlRewriter {
    pub fn new() -> Self {
        Self {
            config: RewriterConfig::default(),
            processed_images: HashMap::new(),
        }
    }
    
    pub fn with_config(config: RewriterConfig) -> Self {
        Self {
            config,
            processed_images: HashMap::new(),
        }
    }
    
    /// Process all images in the DOM, fetching and optionally inlining them
    pub fn process_images(&mut self, dom: &mut Dom, network: &NetworkManager) {
        let refs = crate::parser::html::extract_image_refs(dom);
        
        for img_ref in refs {
            let resolved_url = network.resolve_url(&img_ref.url);
            
            // Skip if already processed
            if self.processed_images.contains_key(&resolved_url) {
                continue;
            }
            
            // Fetch the image
            if let Some(resource) = network.fetch_resource(&resolved_url) {
                let processed = self.process_single_image(&img_ref.url, &resolved_url, &resource);
                self.processed_images.insert(resolved_url.clone(), processed);
            }
        }
        
        // Rewrite the DOM
        self.rewrite_dom(dom);
    }
    
    fn process_single_image(
        &self,
        original_url: &str,
        resolved_url: &str,
        resource: &FetchedResource,
    ) -> ProcessedImage {
        let image_type = detect_image_type(Some(&resource.content_type), &resource.data);
        
        // Determine image dimensions
        let (width, height) = self.get_image_dimensions(&resource.data, image_type);
        
        // Check if we should inline this image
        let should_inline = self.config.inline_images && 
                           resource.data.len() <= self.config.max_inline_size;
        
        let final_url = if should_inline {
            // Try to create a data URI
            match self.create_data_uri(&resource.data, image_type) {
                Some(data_uri) => data_uri,
                None => resolved_url.to_string(),
            }
        } else {
            resolved_url.to_string()
        };
        
        ProcessedImage {
            original_url: original_url.to_string(),
            resolved_url: resolved_url.to_string(),
            final_url,
            inlined: should_inline && resource.data.len() <= self.config.max_inline_size,
            width,
            height,
        }
    }
    
    fn get_image_dimensions(&self, data: &[u8], image_type: ImageType) -> (Option<u32>, Option<u32>) {
        match decode_image(data, image_type, None, None) {
            Ok(img) => (Some(img.width()), Some(img.height())),
            Err(_) => (None, None),
        }
    }
    
    fn create_data_uri(&self, data: &[u8], image_type: ImageType) -> Option<String> {
        // For non-SVG images, we can create a data URI directly
        if image_type == ImageType::Svg {
            // For SVG, encode as text
            let svg_str = std::str::from_utf8(data).ok()?;
            let encoded = encode_uri_component(svg_str);
            Some(format!("data:image/svg+xml,{}", encoded))
        } else {
            // For raster images, use base64
            let base64 = encode_base64(data);
            Some(format!("data:{};base64,{}", image_type.mime_type(), base64))
        }
    }
    
    fn rewrite_dom(&self, dom: &mut Dom) {
        self.rewrite_node(dom, dom.root());
    }
    
    fn rewrite_node(&self, dom: &mut Dom, node_id: NodeId) {
        // Get children first to avoid borrow issues
        let children: Vec<NodeId> = dom.nodes[node_id].children.clone();
        
        // Process this node
        if let NodeType::Element(ref mut el) = dom.nodes[node_id].node_type {
            let tag = el.tag_name.to_lowercase();
            
            match tag.as_str() {
                "img" => self.rewrite_img_element(el),
                "source" => self.rewrite_source_element(el),
                "link" => self.rewrite_link_element(el),
                _ => {}
            }
            
            // Rewrite style attribute
            self.rewrite_style_attribute(el);
        }
        
        // Recurse into children
        for child_id in children {
            self.rewrite_node(dom, child_id);
        }
    }
    
    fn rewrite_img_element(&self, el: &mut ElementData) {
        // Rewrite src attribute
        if let Some(src_idx) = el.attributes.iter().position(|(k, _)| k.eq_ignore_ascii_case("src")) {
            let src = el.attributes[src_idx].1.clone();
            if let Some(processed) = self.find_processed(&src) {
                el.attributes[src_idx].1 = processed.final_url.clone();
            }
        }
        
        // Rewrite srcset attribute
        if let Some(srcset_idx) = el.attributes.iter().position(|(k, _)| k.eq_ignore_ascii_case("srcset")) {
            let srcset = el.attributes[srcset_idx].1.clone();
            let rewritten = self.rewrite_srcset(&srcset);
            el.attributes[srcset_idx].1 = rewritten;
        }
    }
    
    fn rewrite_source_element(&self, el: &mut ElementData) {
        // Rewrite srcset attribute
        if let Some(srcset_idx) = el.attributes.iter().position(|(k, _)| k.eq_ignore_ascii_case("srcset")) {
            let srcset = el.attributes[srcset_idx].1.clone();
            let rewritten = self.rewrite_srcset(&srcset);
            el.attributes[srcset_idx].1 = rewritten;
        }
        
        // Rewrite src attribute
        if let Some(src_idx) = el.attributes.iter().position(|(k, _)| k.eq_ignore_ascii_case("src")) {
            let src = el.attributes[src_idx].1.clone();
            if let Some(processed) = self.find_processed(&src) {
                el.attributes[src_idx].1 = processed.final_url.clone();
            }
        }
    }
    
    fn rewrite_link_element(&self, el: &mut ElementData) {
        let rel = el.attributes.iter()
            .find(|(k, _)| k.eq_ignore_ascii_case("rel"))
            .map(|(_, v)| v.to_lowercase())
            .unwrap_or_default();
        
        if rel.contains("icon") {
            if let Some(href_idx) = el.attributes.iter().position(|(k, _)| k.eq_ignore_ascii_case("href")) {
                let href = el.attributes[href_idx].1.clone();
                if let Some(processed) = self.find_processed(&href) {
                    el.attributes[href_idx].1 = processed.final_url.clone();
                }
            }
        }
    }
    
    fn rewrite_style_attribute(&self, el: &mut ElementData) {
        if let Some(style_idx) = el.attributes.iter().position(|(k, _)| k.eq_ignore_ascii_case("style")) {
            let style = el.attributes[style_idx].1.clone();
            let rewritten = self.rewrite_css_urls(&style);
            el.attributes[style_idx].1 = rewritten;
        }
    }
    
    fn rewrite_srcset(&self, srcset: &str) -> String {
        let mut parts = Vec::new();
        
        for entry in srcset.split(',') {
            let entry = entry.trim();
            if entry.is_empty() {
                continue;
            }
            
            let tokens: Vec<&str> = entry.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }
            
            let url = tokens[0];
            let descriptor = if tokens.len() > 1 { tokens[1] } else { "" };
            
            let rewritten_url = if let Some(processed) = self.find_processed(url) {
                processed.final_url.clone()
            } else {
                url.to_string()
            };
            
            if descriptor.is_empty() {
                parts.push(rewritten_url);
            } else {
                parts.push(format!("{} {}", rewritten_url, descriptor));
            }
        }
        
        parts.join(", ")
    }
    
    fn rewrite_css_urls(&self, css: &str) -> String {
        let mut result = css.to_string();
        let urls = crate::parser::html::parse_css_urls(css);
        
        // Process URLs from end to start to preserve positions
        for url_ref in urls.into_iter().rev() {
            if let Some(processed) = self.find_processed(&url_ref.url) {
                // Replace the URL in the CSS
                result = result.replace(&url_ref.url, &processed.final_url);
            }
        }
        
        result
    }
    
    fn find_processed(&self, url: &str) -> Option<&ProcessedImage> {
        // First try direct match
        if let Some(p) = self.processed_images.get(url) {
            return Some(p);
        }
        
        // Then try to find by original_url
        self.processed_images.values()
            .find(|p| p.original_url == url)
    }
    
    /// Get all processed images
    pub fn processed_images(&self) -> &HashMap<String, ProcessedImage> {
        &self.processed_images
    }
}

impl Default for HtmlRewriter {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple base64 encoder
fn encode_base64(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    let mut result = String::new();
    let mut i = 0;
    
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = data.get(i + 1).copied().unwrap_or(0) as u32;
        let b2 = data.get(i + 2).copied().unwrap_or(0) as u32;
        
        let triple = (b0 << 16) | (b1 << 8) | b2;
        
        result.push(ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        
        if i + 1 < data.len() {
            result.push(ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        if i + 2 < data.len() {
            result.push(ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        
        i += 3;
    }
    
    result
}

/// Simple URI component encoder for SVG data URIs
fn encode_uri_component(s: &str) -> String {
    let mut result = String::new();
    
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '!' | '~' | '*' | '\'' | '(' | ')' => {
                result.push(c);
            }
            _ => {
                for byte in c.to_string().as_bytes() {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_base64() {
        assert_eq!(encode_base64(b"Hello"), "SGVsbG8=");
        assert_eq!(encode_base64(b"Hello!"), "SGVsbG8h");
        assert_eq!(encode_base64(b""), "");
    }
    
    #[test]
    fn test_encode_uri_component() {
        assert_eq!(encode_uri_component("hello"), "hello");
        assert_eq!(encode_uri_component("hello world"), "hello%20world");
        assert_eq!(encode_uri_component("<svg>"), "%3Csvg%3E");
    }
}
