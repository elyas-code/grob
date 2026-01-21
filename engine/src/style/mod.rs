use crate::dom::NodeId;
use crate::dom::{Dom, NodeType};
use std::collections::HashMap;

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
        self.get("font-family").unwrap_or("Arial")
    }

    pub fn get_font_size(&self) -> f32 {
        self.get("font-size")
            .and_then(|s| {
                let s = s.trim();
                if s.ends_with("px") {
                    s.trim_end_matches("px").parse().ok()
                } else if s.ends_with("em") {
                    s.trim_end_matches("em").parse::<f32>().ok().map(|e| e * 24.0)
                } else {
                    s.parse().ok()
                }
            })
            .unwrap_or(24.0)
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
pub enum Selector {
    Tag(String),
    Class(String),
    Id(String),
    TagWithPseudo(String, String),
    Any,
}

pub struct Stylesheet {
    pub rules: Vec<CssRule>,
}

impl Stylesheet {
    pub fn new() -> Self { Self { rules: vec![] } }

    pub fn add_rule(&mut self, selector: Selector, declarations: Style) {
        self.rules.push(CssRule { selector, declarations });
    }

    pub fn compute_style(&self, dom: &Dom, node_id: NodeId) -> Style {
        let node = &dom.nodes[node_id];
        let mut result = Style { properties: HashMap::new() };

        if let NodeType::Element(el) = &node.node_type {
            // Apply default styles for anchors
            if el.tag_name == "a" {
                result.properties.insert("color".to_string(), "#0000ff".to_string());
                result.properties.insert("text-decoration".to_string(), "underline".to_string());
            }

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
                    result.properties.extend(rule.declarations.properties.clone());
                }
            }
        }

        result
    }
}