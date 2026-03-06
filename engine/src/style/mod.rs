use crate::dom::NodeId;
use crate::dom::{Dom, NodeType};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    Mobile,
    Tablet,
    Desktop,
}

impl Breakpoint {
    pub fn from_width(width: f32) -> Self {
        if width < 768.0 {
            Breakpoint::Mobile
        } else if width < 1024.0 {
            Breakpoint::Tablet
        } else {
            Breakpoint::Desktop
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn breakpoint(&self) -> Breakpoint {
        Breakpoint::from_width(self.width)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self { width: 1200.0, height: 800.0 }
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub properties: HashMap<String, String>,
}

impl Style {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.properties.get(key).map(|s| s.as_str())
    }

    pub fn get_font_family(&self) -> &str {
        self.get("font-family").unwrap_or("Times New Roman")
    }

    pub fn get_font_size(&self) -> f32 {
        self.get("font-size")
            .and_then(|s| {
                let s = s.trim();
                if s.ends_with("px") {
                    s.trim_end_matches("px").parse().ok()
                } else if s.ends_with("em") {
                    s.trim_end_matches("em").parse::<f32>().ok().map(|e| e * 16.0)
                } else if s.ends_with("rem") {
                    s.trim_end_matches("rem").parse::<f32>().ok().map(|e| e * 16.0)
                } else {
                    s.parse().ok()
                }
            })
            .unwrap_or(16.0)
    }

    pub fn get_font_weight(&self) -> &str {
        self.get("font-weight").unwrap_or("normal")
    }

    pub fn is_bold(&self) -> bool {
        matches!(self.get_font_weight(), "bold" | "700" | "800" | "900")
    }

    pub fn get_font_style(&self) -> &str {
        self.get("font-style").unwrap_or("normal")
    }

    pub fn is_italic(&self) -> bool {
        self.get_font_style() == "italic"
    }

    pub fn get_color(&self) -> (u8, u8, u8) {
        if let Some(color_str) = self.get("color") {
            parse_color(color_str)
        } else {
            (0, 0, 0)
        }
    }

    pub fn get_background_color(&self) -> Option<(u8, u8, u8)> {
        self.get("background").or_else(|| self.get("background-color")).map(|c| parse_color(c))
    }

    pub fn get_opacity(&self) -> f32 {
        self.get("opacity")
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(1.0)
    }

    pub fn get_text_decoration(&self) -> Option<&str> {
        self.get("text-decoration")
    }

    pub fn has_text_decoration(&self, decoration: &str) -> bool {
        self.get_text_decoration()
            .map(|d| d.contains(decoration))
            .unwrap_or(false)
    }
    
    pub fn get_width_percentage(&self) -> Option<f32> {
        self.get("width")
            .and_then(|s| {
                let s = s.trim();
                if s.ends_with("vw") {
                    // Viewport width percentage
                    s.trim_end_matches("vw").parse::<f32>().ok().map(|v| v / 100.0)
                } else if s.ends_with("%") {
                    // Percentage
                    s.trim_end_matches("%").parse::<f32>().ok().map(|v| v / 100.0)
                } else {
                    None
                }
            })
    }

    /// Parse a spacing value (padding or margin) that can be in px, em, or vh
    /// Note: vh values just return the raw number, must be multiplied by viewport height / 100 by caller
    fn parse_spacing_value(value: &str) -> f32 {
        let s = value.trim();
        if s.ends_with("px") {
            s.trim_end_matches("px").parse::<f32>().unwrap_or(0.0)
        } else if s.ends_with("em") {
            s.trim_end_matches("em").parse::<f32>().unwrap_or(0.0) * 16.0 // 1em = 16px
        } else if s.ends_with("vh") {
            // Return negative to signal it's a vh value, caller will handle
            s.trim_end_matches("vh").parse::<f32>().unwrap_or(0.0)
        } else {
            s.parse::<f32>().unwrap_or(0.0)
        }
    }

    /// Parse a spacing value with viewport awareness
    fn parse_spacing_value_with_viewport(value: &str, viewport_height: f32) -> f32 {
        let s = value.trim();
        if s.ends_with("px") {
            s.trim_end_matches("px").parse::<f32>().unwrap_or(0.0)
        } else if s.ends_with("em") {
            s.trim_end_matches("em").parse::<f32>().unwrap_or(0.0) * 16.0 // 1em = 16px
        } else if s.ends_with("vh") {
            // vh = viewport height percentage
            let vh_value = s.trim_end_matches("vh").parse::<f32>().unwrap_or(0.0);
            viewport_height * vh_value / 100.0
        } else if s.ends_with("vw") {
            // vw = viewport width percentage - caller must handle this separately
            // For now just parse the number
            s.trim_end_matches("vw").parse::<f32>().unwrap_or(0.0)
        } else {
            s.parse::<f32>().unwrap_or(0.0)
        }
    }

    /// Get padding value (top, right, bottom, left) - returns all four values
    pub fn get_padding(&self) -> (f32, f32, f32, f32) {
        if let Some(padding_str) = self.get("padding") {
            let parts: Vec<&str> = padding_str.split_whitespace().collect();
            match parts.len() {
                1 => {
                    let val = Self::parse_spacing_value(parts[0]);
                    (val, val, val, val) // all sides same
                }
                2 => {
                    let v1 = Self::parse_spacing_value(parts[0]);
                    let v2 = Self::parse_spacing_value(parts[1]);
                    (v1, v2, v1, v2) // top/bottom, left/right
                }
                3 => {
                    let top = Self::parse_spacing_value(parts[0]);
                    let h = Self::parse_spacing_value(parts[1]);
                    let bottom = Self::parse_spacing_value(parts[2]);
                    (top, h, bottom, h)
                }
                _ => {
                    let top = Self::parse_spacing_value(parts[0]);
                    let right = Self::parse_spacing_value(parts[1]);
                    let bottom = Self::parse_spacing_value(parts[2]);
                    let left = Self::parse_spacing_value(parts[3]);
                    (top, right, bottom, left)
                }
            }
        } else {
            (0.0, 0.0, 0.0, 0.0)
        }
    }

    /// Get margin value (top, right, bottom, left) - returns all four values
    /// Supports both shorthand `margin` and individual `margin-top`, `margin-right`, etc.
    pub fn get_margin(&self) -> (f32, f32, f32, f32) {
        let mut top = 0.0_f32;
        let mut right = 0.0_f32;
        let mut bottom = 0.0_f32;
        let mut left = 0.0_f32;

        // First check shorthand margin
        if let Some(margin_str) = self.get("margin") {
            let parts: Vec<&str> = margin_str.split_whitespace().collect();
            match parts.len() {
                1 => {
                    let val = Self::parse_margin_value(parts[0]);
                    top = val; right = val; bottom = val; left = val;
                }
                2 => {
                    let v1 = Self::parse_margin_value(parts[0]);
                    let v2 = Self::parse_margin_value(parts[1]);
                    top = v1; bottom = v1; right = v2; left = v2;
                }
                3 => {
                    top = Self::parse_margin_value(parts[0]);
                    right = Self::parse_margin_value(parts[1]);
                    left = right;
                    bottom = Self::parse_margin_value(parts[2]);
                }
                _ => {
                    top = Self::parse_margin_value(parts[0]);
                    right = Self::parse_margin_value(parts[1]);
                    bottom = Self::parse_margin_value(parts[2]);
                    left = Self::parse_margin_value(parts[3]);
                }
            }
        }

        // Override with individual properties if present
        if let Some(mt) = self.get("margin-top") {
            top = Self::parse_margin_value(mt);
        }
        if let Some(mr) = self.get("margin-right") {
            right = Self::parse_margin_value(mr);
        }
        if let Some(mb) = self.get("margin-bottom") {
            bottom = Self::parse_margin_value(mb);
        }
        if let Some(ml) = self.get("margin-left") {
            left = Self::parse_margin_value(ml);
        }

        (top, right, bottom, left)
    }

    /// Get margin value (top, right, bottom, left) with viewport unit support
    /// vh values are converted to pixels using viewport_height
    pub fn get_margin_with_viewport(&self, viewport_height: f32) -> (f32, f32, f32, f32) {
        let mut top = 0.0_f32;
        let mut right = 0.0_f32;
        let mut bottom = 0.0_f32;
        let mut left = 0.0_f32;

        // First check shorthand margin
        if let Some(margin_str) = self.get("margin") {
            let parts: Vec<&str> = margin_str.split_whitespace().collect();
            match parts.len() {
                1 => {
                    let val = Self::parse_margin_value_with_viewport(parts[0], viewport_height);
                    top = val; right = val; bottom = val; left = val;
                }
                2 => {
                    let v1 = Self::parse_margin_value_with_viewport(parts[0], viewport_height);
                    let v2 = Self::parse_margin_value_with_viewport(parts[1], viewport_height);
                    top = v1; bottom = v1; right = v2; left = v2;
                }
                3 => {
                    top = Self::parse_margin_value_with_viewport(parts[0], viewport_height);
                    right = Self::parse_margin_value_with_viewport(parts[1], viewport_height);
                    left = right;
                    bottom = Self::parse_margin_value_with_viewport(parts[2], viewport_height);
                }
                _ => {
                    top = Self::parse_margin_value_with_viewport(parts[0], viewport_height);
                    right = Self::parse_margin_value_with_viewport(parts[1], viewport_height);
                    bottom = Self::parse_margin_value_with_viewport(parts[2], viewport_height);
                    left = Self::parse_margin_value_with_viewport(parts[3], viewport_height);
                }
            }
        }

        // Override with individual properties if present
        if let Some(mt) = self.get("margin-top") {
            top = Self::parse_margin_value_with_viewport(mt, viewport_height);
        }
        if let Some(mr) = self.get("margin-right") {
            right = Self::parse_margin_value_with_viewport(mr, viewport_height);
        }
        if let Some(mb) = self.get("margin-bottom") {
            bottom = Self::parse_margin_value_with_viewport(mb, viewport_height);
        }
        if let Some(ml) = self.get("margin-left") {
            left = Self::parse_margin_value_with_viewport(ml, viewport_height);
        }

        (top, right, bottom, left)
    }

    /// Check if element has auto horizontal margin (for centering)
    pub fn has_auto_horizontal_margin(&self) -> bool {
        // Check individual properties first
        let left_auto = self.get("margin-left").map(|s| s.trim() == "auto").unwrap_or(false);
        let right_auto = self.get("margin-right").map(|s| s.trim() == "auto").unwrap_or(false);
        
        if left_auto && right_auto {
            return true;
        }

        // Check shorthand margin
        if let Some(margin_str) = self.get("margin") {
            let parts: Vec<&str> = margin_str.split_whitespace().collect();
            match parts.len() {
                2 => {
                    // margin: vertical horizontal
                    return parts[1].trim() == "auto";
                }
                4 => {
                    // margin: top right bottom left
                    return parts[1].trim() == "auto" && parts[3].trim() == "auto";
                }
                _ => {}
            }
        }
        false
    }

    /// Parse margin value, returning 0 for "auto" (to be handled by layout engine)
    fn parse_margin_value(value: &str) -> f32 {
        let s = value.trim();
        if s == "auto" {
            0.0 // Layout engine will handle auto margin
        } else {
            Self::parse_spacing_value(s)
        }
    }

    /// Parse margin value with viewport unit support, returning 0 for "auto"
    fn parse_margin_value_with_viewport(value: &str, viewport_height: f32) -> f32 {
        let s = value.trim();
        if s == "auto" {
            0.0 // Layout engine will handle auto margin
        } else {
            Self::parse_spacing_value_with_viewport(s, viewport_height)
        }
    }

    pub fn get_width_px(&self, viewport_width: f32) -> Option<f32> {
        self.get("width").and_then(|s| {
            let s = s.trim();
            if s.ends_with("vw") {
                s.trim_end_matches("vw").parse::<f32>().ok().map(|v| viewport_width * v / 100.0)
            } else if s.ends_with("%") {
                s.trim_end_matches("%").parse::<f32>().ok().map(|v| viewport_width * v / 100.0)
            } else if s.ends_with("px") {
                s.trim_end_matches("px").parse::<f32>().ok()
            } else {
                s.parse::<f32>().ok()
            }
        })
    }

    pub fn get_max_width_px(&self, viewport_width: f32) -> Option<f32> {
        self.get("max-width").and_then(|s| {
            let s = s.trim();
            if s.ends_with("vw") {
                s.trim_end_matches("vw").parse::<f32>().ok().map(|v| viewport_width * v / 100.0)
            } else if s.ends_with("%") {
                s.trim_end_matches("%").parse::<f32>().ok().map(|v| viewport_width * v / 100.0)
            } else if s.ends_with("px") {
                s.trim_end_matches("px").parse::<f32>().ok()
            } else {
                s.parse::<f32>().ok()
            }
        })
    }
}

impl Default for Stylesheet {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_color(color: &str) -> (u8, u8, u8) {
    let color = color.trim();
    if color.starts_with('#') {
        let hex = &color[1..];
        if hex.len() == 3 {
            let r = u8::from_str_radix(&format!("{}{}", &hex[0..1], &hex[0..1]), 16).unwrap_or(0);
            let g = u8::from_str_radix(&format!("{}{}", &hex[1..2], &hex[1..2]), 16).unwrap_or(0);
            let b = u8::from_str_radix(&format!("{}{}", &hex[2..3], &hex[2..3]), 16).unwrap_or(0);
            (r, g, b)
        } else if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            (r, g, b)
        } else {
            (0, 0, 0)
        }
    } else {
        (0, 0, 0)
    }
}

#[derive(Debug, Clone)]
pub struct CssRule {
    pub selector: Selector,
    pub declarations: Style,
}

#[derive(Debug, Clone)]
pub struct MediaRule {
    pub condition: MediaCondition,
    pub rules: Vec<CssRule>,
}

#[derive(Debug, Clone)]
pub enum MediaCondition {
    MinWidth(f32),
    MaxWidth(f32),
    Breakpoint(Breakpoint),
}

impl MediaCondition {
    pub fn matches(&self, viewport: &Viewport) -> bool {
        match self {
            MediaCondition::MinWidth(min) => viewport.width >= *min,
            MediaCondition::MaxWidth(max) => viewport.width <= *max,
            MediaCondition::Breakpoint(bp) => viewport.breakpoint() == *bp,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Selector {
    Tag(String),
    Class(String),
    Id(String),
    TagWithPseudo(String, String),
    Any,
}

pub struct Stylesheet {
    pub rules: Vec<CssRule>,
    pub media_rules: Vec<MediaRule>,
    viewport: Viewport,
}

impl Stylesheet {
    pub fn new() -> Self { 
        Self { 
            rules: vec![], 
            media_rules: vec![],
            viewport: Viewport::default(),
        } 
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    pub fn get_viewport(&self) -> Viewport {
        self.viewport
    }

    pub fn add_rule(&mut self, selector: Selector, declarations: Style) {
        self.rules.push(CssRule { selector, declarations });
    }

    pub fn add_media_rule(&mut self, condition: MediaCondition, rules: Vec<CssRule>) {
        self.media_rules.push(MediaRule { condition, rules });
    }

    pub fn compute_style(&self, dom: &Dom, node_id: NodeId) -> Style {
        self.compute_style_with_viewport(dom, node_id, &self.viewport)
    }

    pub fn compute_style_with_viewport(&self, dom: &Dom, node_id: NodeId, viewport: &Viewport) -> Style {
        let node = &dom.nodes[node_id];
        let mut result = Style { properties: HashMap::new() };

        if let NodeType::Element(el) = &node.node_type {
            // Step 0: Inherit inheritable properties from parent element
            if let Some(parent_id) = node.parent {
                let parent_style = self.compute_style_with_viewport(dom, parent_id, viewport);
                // Inherit inheritable CSS properties
                for (key, value) in &parent_style.properties {
                    if self.is_inheritable_property(key) {
                        result.properties.insert(key.clone(), value.clone());
                    }
                }
            }

            // Step 1: Apply default user agent styles for this element type
            self.apply_default_styles(&mut result, &el.tag_name);

            // Step 2: Iterate through custom CSS rules and apply matching ones
            // If a property exists in the custom rule, it overrides the default
            // If a property doesn't exist in the custom rule, the default is kept
            for rule in &self.rules {
                let matches = match &rule.selector {
                    Selector::Tag(tag) if tag == "*" => true,
                    Selector::Tag(tag) if tag == &el.tag_name => true,
                    Selector::Id(id) => el.attributes.iter().any(|(k, v)| k == "id" && v == id),
                    Selector::Class(class) => el.attributes.iter().any(|(k, v)| k == "class" && v == class),
                    Selector::TagWithPseudo(tag, _pseudo) => tag == &el.tag_name,
                    Selector::Any => true,
                    _ => false,
                };

                if matches {
                    // Apply custom rule properties - these override defaults
                    for (key, value) in &rule.declarations.properties {
                        result.properties.insert(key.clone(), value.clone());
                    }
                }
            }

            // Step 3: Apply matching media query rules based on viewport
            for media_rule in &self.media_rules {
                if media_rule.condition.matches(viewport) {
                    for rule in &media_rule.rules {
                        let media_matches = match &rule.selector {
                            Selector::Tag(tag) if tag == "*" => true,
                            Selector::Tag(tag) if tag == &el.tag_name => true,
                            Selector::Id(id) => el.attributes.iter().any(|(k, v)| k == "id" && v == id),
                            Selector::Class(class) => el.attributes.iter().any(|(k, v)| k == "class" && v == class),
                            Selector::TagWithPseudo(tag, _pseudo) => tag == &el.tag_name,
                            Selector::Any => true,
                            _ => false,
                        };

                        if media_matches {
                            for (key, value) in &rule.declarations.properties {
                                result.properties.insert(key.clone(), value.clone());
                            }
                        }
                    }
                }
            }
        } else if let NodeType::Text(_text) = &node.node_type {
            // Text nodes inherit styles from their parent element
            if let Some(parent_id) = node.parent {
                result = self.compute_style_with_viewport(dom, parent_id, viewport);
            }
        }

        result
    }

    /// Check if a CSS property is inheritable
    fn is_inheritable_property(&self, property: &str) -> bool {
        match property {
            "font-family" | "font-size" | "font-weight" | "font-style" | 
            "color" | "line-height" | "text-align" | "text-decoration" |
            "font-variant" | "letter-spacing" | "word-spacing" => true,
            _ => false,
        }
    }

    /// Apply default user agent styles for each element type
    fn apply_default_styles(&self, style: &mut Style, tag_name: &str) {
        match tag_name {
            // Hyperlink
            "a" => {
                style.properties.insert("color".to_string(), "#0000ff".to_string());
                style.properties.insert("text-decoration".to_string(), "underline".to_string());
            }
            // Text formatting
            "b" | "strong" => {
                style.properties.insert("font-weight".to_string(), "bold".to_string());
            }
            "i" | "em" => {
                style.properties.insert("font-style".to_string(), "italic".to_string());
            }
            "u" => {
                style.properties.insert("text-decoration".to_string(), "underline".to_string());
            }
            "s" | "del" => {
                style.properties.insert("text-decoration".to_string(), "line-through".to_string());
            }
            "code" => {
                style.properties.insert("font-family".to_string(), "monospace".to_string());
            }
            "pre" => {
                style.properties.insert("font-family".to_string(), "monospace".to_string());
                style.properties.insert("margin".to_string(), "1em 0.5em".to_string());
            }
            // Headings - per HTML spec default margins
            "h1" => {
                style.properties.insert("font-size".to_string(), "2em".to_string());
                style.properties.insert("font-weight".to_string(), "bold".to_string());
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
            }
            "h2" => {
                style.properties.insert("font-size".to_string(), "1.5em".to_string());
                style.properties.insert("font-weight".to_string(), "bold".to_string());
                style.properties.insert("margin".to_string(), "0.25em 0.5em".to_string());
            }
            "h3" => {
                style.properties.insert("font-size".to_string(), "1.17em".to_string());
                style.properties.insert("font-weight".to_string(), "bold".to_string());
                style.properties.insert("margin".to_string(), "0.2em 0.5em".to_string());
            }
            "h4" => {
                style.properties.insert("font-size".to_string(), "1em".to_string());
                style.properties.insert("font-weight".to_string(), "bold".to_string());
                style.properties.insert("margin".to_string(), "0.2em 0.5em".to_string());
            }
            "h5" => {
                style.properties.insert("font-size".to_string(), "0.83em".to_string());
                style.properties.insert("font-weight".to_string(), "bold".to_string());
                style.properties.insert("margin".to_string(), "0.2em 0.5em".to_string());
            }
            "h6" => {
                style.properties.insert("font-size".to_string(), "0.67em".to_string());
                style.properties.insert("font-weight".to_string(), "bold".to_string());
                style.properties.insert("margin".to_string(), "0.2em 0.5em".to_string());
            }
            // Paragraph
            "p" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
            }
            // Lists - with horizontal margins and left padding for indentation
            "ul" | "ol" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
                style.properties.insert("padding-left".to_string(), "40px".to_string());
            }
            "li" => {
                // li typically has no margin by default, inherits from ul/ol
                style.properties.insert("margin".to_string(), "0".to_string());
            }
            "dl" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
            }
            "dt" => {
                // Definition term - bold
                style.properties.insert("font-weight".to_string(), "bold".to_string());
            }
            "dd" => {
                // Definition data - indented with left margin
                style.properties.insert("margin-left".to_string(), "2em".to_string());
            }
            // Block elements - blockquote with left margin
            "blockquote" => {
                style.properties.insert("margin".to_string(), "0.3em 0 0.3em 2em".to_string());
            }
            "hr" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
                style.properties.insert("border".to_string(), "1px solid #ccc".to_string());
            }
            "address" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
                style.properties.insert("font-style".to_string(), "italic".to_string());
            }
            // HTML5 semantic elements - minimal/no margins
            "article" | "aside" | "section" | "header" | "footer" | "nav" | "main" => {
                style.properties.insert("margin".to_string(), "0".to_string());
            }
            "figure" => {
                // Figures have left/right margins for visual separation
                style.properties.insert("margin".to_string(), "0.3em 2em".to_string());
            }
            "figcaption" => {
                style.properties.insert("font-style".to_string(), "italic".to_string());
                style.properties.insert("margin".to_string(), "0".to_string());
            }
            // Form elements
            "form" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
            }
            "fieldset" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
                style.properties.insert("padding".to_string(), "0.5em".to_string());
                style.properties.insert("border".to_string(), "1px solid #ccc".to_string());
            }
            "legend" => {
                style.properties.insert("padding".to_string(), "0 0.25em".to_string());
            }
            // Table elements
            "table" => {
                style.properties.insert("margin".to_string(), "0.3em 0.5em".to_string());
                style.properties.insert("border-collapse".to_string(), "collapse".to_string());
            }
            // Body element
            "body" => {
                style.properties.insert("margin".to_string(), "8px".to_string());
            }
            _ => {
                // No specific defaults for other elements
            }
        }
    }
}