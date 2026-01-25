// Image reference extraction from HTML
//
// This module provides functions to extract all image references from HTML:
// - <img src> and <img srcset>
// - <picture><source srcset>
// - <link rel="icon">
// - CSS url() in style attributes and <style> tags
// - background-image in inline styles

use crate::dom::{Dom, NodeId, NodeType};

/// Represents an image reference found in HTML
#[derive(Debug, Clone, PartialEq)]
pub struct ImageRef {
    /// The URL of the image (may be relative)
    pub url: String,
    /// The type of reference
    pub ref_type: ImageRefType,
    /// Associated media query (for <source media="...">)
    pub media: Option<String>,
    /// Size hints (for srcset width descriptors)
    pub sizes: Option<String>,
    /// The node ID where this reference was found
    pub node_id: NodeId,
}

/// Types of image references
#[derive(Debug, Clone, PartialEq)]
pub enum ImageRefType {
    /// <img src="...">
    ImgSrc,
    /// <img srcset="..."> or <source srcset="...">
    Srcset { descriptors: Vec<SrcsetDescriptor> },
    /// <link rel="icon"> or <link rel="shortcut icon">
    Favicon,
    /// <link rel="apple-touch-icon">
    TouchIcon,
    /// CSS url() in style attribute or stylesheet
    CssUrl { property: String },
    /// <source> within <picture>
    PictureSource,
}

/// Srcset descriptor
#[derive(Debug, Clone, PartialEq)]
pub struct SrcsetDescriptor {
    pub url: String,
    pub width: Option<u32>,
    pub density: Option<f32>,
}

/// Extract the <base href> value from the DOM, if present
pub fn extract_base_href(dom: &Dom) -> Option<String> {
    fn find_base(dom: &Dom, node_id: NodeId) -> Option<String> {
        let node = &dom.nodes[node_id];
        
        if let NodeType::Element(el) = &node.node_type {
            if el.tag_name.eq_ignore_ascii_case("base") {
                for (key, value) in &el.attributes {
                    if key.eq_ignore_ascii_case("href") && !value.is_empty() {
                        return Some(value.clone());
                    }
                }
            }
        }
        
        for &child_id in &node.children {
            if let Some(href) = find_base(dom, child_id) {
                return Some(href);
            }
        }
        
        None
    }
    
    find_base(dom, dom.root())
}

/// Extract all image references from the DOM
pub fn extract_image_refs(dom: &Dom) -> Vec<ImageRef> {
    let mut refs = Vec::new();
    extract_from_node(dom, dom.root(), &mut refs);
    refs
}

fn extract_from_node(dom: &Dom, node_id: NodeId, refs: &mut Vec<ImageRef>) {
    let node = &dom.nodes[node_id];
    
    if let NodeType::Element(el) = &node.node_type {
        let tag = el.tag_name.to_lowercase();
        
        match tag.as_str() {
            "img" => {
                extract_img_refs(el, node_id, refs);
            }
            "source" => {
                extract_source_refs(el, node_id, refs);
            }
            "link" => {
                extract_link_refs(el, node_id, refs);
            }
            _ => {}
        }
        
        // Check for style attribute with background-image
        if let Some(style) = get_attribute(el, "style") {
            extract_css_url_refs(&style, node_id, refs);
        }
    }
    
    // Recurse into children
    for &child_id in &node.children {
        extract_from_node(dom, child_id, refs);
    }
}

fn get_attribute(el: &crate::dom::ElementData, name: &str) -> Option<String> {
    el.attributes.iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.clone())
}

fn extract_img_refs(el: &crate::dom::ElementData, node_id: NodeId, refs: &mut Vec<ImageRef>) {
    // Extract src
    if let Some(src) = get_attribute(el, "src") {
        if !src.is_empty() {
            refs.push(ImageRef {
                url: src,
                ref_type: ImageRefType::ImgSrc,
                media: None,
                sizes: get_attribute(el, "sizes"),
                node_id,
            });
        }
    }
    
    // Extract srcset
    if let Some(srcset) = get_attribute(el, "srcset") {
        let descriptors = parse_srcset_attribute(&srcset);
        if !descriptors.is_empty() {
            refs.push(ImageRef {
                url: descriptors[0].url.clone(), // Primary URL
                ref_type: ImageRefType::Srcset { descriptors },
                media: None,
                sizes: get_attribute(el, "sizes"),
                node_id,
            });
        }
    }
}

fn extract_source_refs(el: &crate::dom::ElementData, node_id: NodeId, refs: &mut Vec<ImageRef>) {
    // <source> can have srcset and media
    if let Some(srcset) = get_attribute(el, "srcset") {
        let descriptors = parse_srcset_attribute(&srcset);
        if !descriptors.is_empty() {
            refs.push(ImageRef {
                url: descriptors[0].url.clone(),
                ref_type: ImageRefType::PictureSource,
                media: get_attribute(el, "media"),
                sizes: get_attribute(el, "sizes"),
                node_id,
            });
        }
    }
    
    // Also check src (for <source> in <video>/<audio> but might appear in <picture>)
    if let Some(src) = get_attribute(el, "src") {
        if !src.is_empty() {
            refs.push(ImageRef {
                url: src,
                ref_type: ImageRefType::PictureSource,
                media: get_attribute(el, "media"),
                sizes: None,
                node_id,
            });
        }
    }
}

fn extract_link_refs(el: &crate::dom::ElementData, node_id: NodeId, refs: &mut Vec<ImageRef>) {
    let rel = get_attribute(el, "rel").unwrap_or_default().to_lowercase();
    let href = get_attribute(el, "href");
    
    if let Some(href) = href {
        if href.is_empty() {
            return;
        }
        
        if rel.contains("icon") {
            if rel.contains("apple-touch") {
                refs.push(ImageRef {
                    url: href,
                    ref_type: ImageRefType::TouchIcon,
                    media: None,
                    sizes: get_attribute(el, "sizes"),
                    node_id,
                });
            } else {
                refs.push(ImageRef {
                    url: href,
                    ref_type: ImageRefType::Favicon,
                    media: None,
                    sizes: get_attribute(el, "sizes"),
                    node_id,
                });
            }
        }
    }
}

fn extract_css_url_refs(style: &str, node_id: NodeId, refs: &mut Vec<ImageRef>) {
    for url_ref in parse_css_urls(style) {
        refs.push(ImageRef {
            url: url_ref.url,
            ref_type: ImageRefType::CssUrl { property: url_ref.property },
            media: None,
            sizes: None,
            node_id,
        });
    }
}

/// Parse srcset attribute into descriptors
pub fn parse_srcset_attribute(srcset: &str) -> Vec<SrcsetDescriptor> {
    let mut descriptors = Vec::new();
    
    for entry in srcset.split(',') {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = entry.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }
        
        let url = parts[0].to_string();
        let mut width = None;
        let mut density = None;
        
        if parts.len() > 1 {
            let desc = parts[1].to_lowercase();
            if desc.ends_with('w') {
                width = desc.trim_end_matches('w').parse().ok();
            } else if desc.ends_with('x') {
                density = desc.trim_end_matches('x').parse().ok();
            }
        }
        
        descriptors.push(SrcsetDescriptor { url, width, density });
    }
    
    descriptors
}

/// Represents a CSS url() reference
#[derive(Debug, Clone)]
pub struct CssUrlRef {
    pub url: String,
    pub property: String,
}

/// Parse CSS for url() references
pub fn parse_css_urls(css: &str) -> Vec<CssUrlRef> {
    let mut refs = Vec::new();
    let css_lower = css.to_lowercase();
    
    // Properties that commonly contain images
    let image_properties = [
        "background",
        "background-image",
        "list-style-image",
        "border-image",
        "border-image-source",
        "mask",
        "mask-image",
        "cursor",
        "content",
    ];
    
    // Find url() patterns
    let mut pos = 0;
    while let Some(url_start) = css_lower[pos..].find("url(") {
        let absolute_start = pos + url_start;
        
        // Find the closing paren
        let after_url = &css[absolute_start + 4..];
        let url_end = find_url_end(after_url);
        
        if let Some(end_pos) = url_end {
            let url_content = &after_url[..end_pos];
            let url = parse_url_value(url_content);
            
            if !url.is_empty() && !url.starts_with("data:") {
                // Try to find the property name
                let property = find_property_name(&css[..absolute_start], &image_properties);
                
                refs.push(CssUrlRef {
                    url,
                    property: property.unwrap_or_else(|| "background-image".to_string()),
                });
            }
            
            pos = absolute_start + 4 + end_pos;
        } else {
            pos = absolute_start + 4;
        }
    }
    
    refs
}

fn find_url_end(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = ' ';
    
    for (i, c) in s.chars().enumerate() {
        if in_string {
            if c == string_char && !s[..i].ends_with('\\') {
                in_string = false;
            }
            continue;
        }
        
        match c {
            '"' | '\'' => {
                if depth == 0 {
                    in_string = true;
                    string_char = c;
                }
            }
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    
    None
}

fn parse_url_value(value: &str) -> String {
    let value = value.trim();
    
    // Remove quotes
    let value = if (value.starts_with('"') && value.ends_with('"')) ||
                   (value.starts_with('\'') && value.ends_with('\'')) {
        &value[1..value.len()-1]
    } else {
        value
    };
    
    value.to_string()
}

fn find_property_name(before: &str, properties: &[&str]) -> Option<String> {
    // Look backwards for a property name
    let before_lower = before.to_lowercase();
    
    for prop in properties {
        if let Some(pos) = before_lower.rfind(prop) {
            // Make sure it's actually a property (followed by :)
            let after_prop = &before[pos + prop.len()..];
            let after_trimmed = after_prop.trim_start();
            if after_trimmed.starts_with(':') {
                return Some(prop.to_string());
            }
        }
    }
    
    None
}

/// Extract all CSS from <style> tags in the DOM
pub fn extract_stylesheets(dom: &Dom) -> Vec<(NodeId, String)> {
    let mut styles = Vec::new();
    extract_styles_from_node(dom, dom.root(), &mut styles);
    styles
}

fn extract_styles_from_node(dom: &Dom, node_id: NodeId, styles: &mut Vec<(NodeId, String)>) {
    let node = &dom.nodes[node_id];
    
    if let NodeType::Element(el) = &node.node_type {
        if el.tag_name.eq_ignore_ascii_case("style") {
            // Get text content
            let mut css = String::new();
            for &child_id in &node.children {
                if let NodeType::Text(text) = &dom.nodes[child_id].node_type {
                    css.push_str(text);
                }
            }
            if !css.is_empty() {
                styles.push((node_id, css));
            }
        }
    }
    
    for &child_id in &node.children {
        extract_styles_from_node(dom, child_id, styles);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_srcset() {
        let srcset = "small.jpg 300w, medium.jpg 600w, large.jpg 1200w";
        let descriptors = parse_srcset_attribute(srcset);
        
        assert_eq!(descriptors.len(), 3);
        assert_eq!(descriptors[0].url, "small.jpg");
        assert_eq!(descriptors[0].width, Some(300));
        assert_eq!(descriptors[1].url, "medium.jpg");
        assert_eq!(descriptors[1].width, Some(600));
    }
    
    #[test]
    fn test_parse_srcset_density() {
        let srcset = "normal.jpg 1x, retina.jpg 2x";
        let descriptors = parse_srcset_attribute(srcset);
        
        assert_eq!(descriptors.len(), 2);
        assert_eq!(descriptors[0].density, Some(1.0));
        assert_eq!(descriptors[1].density, Some(2.0));
    }
    
    #[test]
    fn test_parse_css_urls() {
        let css = "background-image: url('image.png'); background: url(other.jpg)";
        let refs = parse_css_urls(css);
        
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].url, "image.png");
        assert_eq!(refs[0].property, "background-image");
        assert_eq!(refs[1].url, "other.jpg");
    }
    
    #[test]
    fn test_parse_css_url_with_quotes() {
        let css = r#"background: url("test.png")"#;
        let refs = parse_css_urls(css);
        
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].url, "test.png");
    }
    
    #[test]
    fn test_extract_img_from_dom() {
        use crate::parser::html::tree_builder::HtmlParser;
        
        // Use proper HTML with DOCTYPE for correct parsing
        let html = r#"<!DOCTYPE html><html><head></head><body><img src="photo.jpg" alt="A photo"></body></html>"#;
        let dom = HtmlParser::new(html).parse();
        
        let refs = extract_image_refs(&dom);
        
        assert!(!refs.is_empty(), "Should find at least one image reference");
        assert_eq!(refs[0].url, "photo.jpg");
        assert!(matches!(refs[0].ref_type, ImageRefType::ImgSrc));
    }
}
