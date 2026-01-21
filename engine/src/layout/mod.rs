// engine/src/layout/mod.rs
use crate::dom::{Dom, NodeId};
use crate::style::{Stylesheet, Style};

#[derive(Debug)]
pub enum BoxType {
    Block,
    Inline,
}

#[derive(Debug)]
pub struct Dimensions {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug)]
pub struct LayoutBox {
    pub node_id: NodeId,
    pub box_type: BoxType,
    pub dimensions: Dimensions,
    pub style: Style,
    pub children: Vec<LayoutBox>,
    pub text_content: Option<String>,
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self {
        Self
    }
    
    /// Measure the width of a space character
    /// Uses heuristic: space is typically ~25% of font-size for most fonts
    /// In future, could query actual font metrics
    fn get_space_width(&self, font_size: f32) -> f32 {
        // Empirically, space width is ~25% of font-size
        // This is more accurate than the previous 0.3 * font_size
        font_size * 0.25
    }
    
    fn measure_word_width(
        &self,
        word: &str,
        font_size: f32,
        _font_family: &str,
    ) -> f32 {
        // Heuristic: ~0.55 * font_size per character
        // More accurate than previous 0.6, accounts for proportional fonts
        word.len() as f32 * font_size * 0.55
    }

    pub fn layout_with_viewport(&self, dom: &Dom, stylesheet: &Stylesheet, viewport_width: f32) -> LayoutBox {
        let root_id = dom.root();
        let exclude_tags = ["head", "meta", "link", "title", "style", "script", "base", "noscript"];
        self.layout_block_container(dom, stylesheet, root_id, 0.0, 0.0, viewport_width, &exclude_tags)
    }

    fn is_block_element(&self, dom: &Dom, node_id: NodeId) -> bool {
        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Text(_) => false,
            crate::dom::NodeType::Element(el) => matches!(el.tag_name.as_str(),
                "p" | "div" | "body" | "html" | "document" | "head" | "title" | "meta" | "link" | 
                "style" | "script" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "img" | "iframe" | 
                "video" | "canvas"),
        }
    }

    fn layout_block_container(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        node_id: NodeId,
        x: f32,
        y: f32,
        width: f32,
        exclude_tags: &[&str],
    ) -> LayoutBox {
        let style = stylesheet.compute_style(dom, node_id);
        
        // Apply width constraints from CSS if specified
        let effective_width = if let Some(width_fraction) = style.get_width_percentage() {
            (width * width_fraction).max(width * 0.1) // At least 10% of available width
        } else {
            width
        };
        
        let mut children_boxes = Vec::new();
        let mut current_y = y;
        let children = dom.nodes[node_id].children.clone();
        let mut child_idx = 0;

        while child_idx < children.len() {
            let child_id = children[child_idx];
            let should_exclude = if let crate::dom::NodeType::Element(el) = &dom.nodes[child_id].node_type {
                exclude_tags.contains(&el.tag_name.as_str())
            } else {
                false
            };

            if should_exclude {
                child_idx += 1;
                continue;
            }

            if self.is_block_element(dom, child_id) {
                // Block element
                let child_box = self.layout_block_container(dom, stylesheet, child_id, x, current_y, effective_width, exclude_tags);
                current_y += child_box.dimensions.height;
                children_boxes.push(child_box);
                child_idx += 1;
            } else {
                // Inline or text - collect all consecutive inline/text children
                let mut inline_children = vec![child_id];
                child_idx += 1;
                
                while child_idx < children.len() {
                    let next_id = children[child_idx];
                    let is_excluded = if let crate::dom::NodeType::Element(el) = &dom.nodes[next_id].node_type {
                        exclude_tags.contains(&el.tag_name.as_str())
                    } else {
                        false
                    };
                    
                    if is_excluded {
                        child_idx += 1;
                        continue;
                    }
                    
                    if self.is_block_element(dom, next_id) {
                        break;
                    }
                    inline_children.push(next_id);
                    child_idx += 1;
                }

                // Layout inline children as a line
                let line_box = self.layout_inline_line(dom, stylesheet, &inline_children, x, current_y, effective_width, exclude_tags);
                current_y += line_box.dimensions.height;
                children_boxes.push(line_box);
            }
        }

        let total_height = current_y - y;
        LayoutBox {
            node_id,
            box_type: BoxType::Block,
            dimensions: Dimensions { x, y, width: effective_width, height: total_height.max(16.0) },
            style,
            children: children_boxes,
            text_content: None,
        }
    }

    fn layout_inline_line(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        inline_children: &[NodeId],
        x: f32,
        mut y: f32,
        width: f32,
        exclude_tags: &[&str],
    ) -> LayoutBox {
        let mut line_boxes = Vec::new();
        let mut current_x = x;
        let mut max_height = 0.0_f32;
        let mut total_height = 0.0_f32;

        for &child_id in inline_children {
            // Handle text nodes specially - split into words for proper wrapping
            if let crate::dom::NodeType::Text(text) = &dom.nodes[child_id].node_type {
                let style = stylesheet.compute_style(dom, child_id);
                let font_size = style.get_font_size();
                let font_family = style.get_font_family();
                let line_height = font_size * 1.2;
                
                // Split text into words (by whitespace - this collapses multiple spaces)
                // Correct behavior for CSS white-space: normal (the default)
                let words: Vec<&str> = text.split_whitespace().collect();
                
                for word in words {
                    // Measure word width using heuristic
                    let word_width = self.measure_word_width(&word, font_size, &font_family);
                    // Space width is now better calibrated (~25% vs previous 30%)
                    let space_width = self.get_space_width(font_size);
                    
                    // Check if word fits on current line
                    if current_x + word_width > x + width && current_x > x {
                        // Word doesn't fit - wrap to next line
                        total_height += max_height;
                        y = y + max_height;
                        current_x = x;
                        max_height = 0.0;
                    }
                    
                    // Add word box
                    let word_box = LayoutBox {
                        node_id: child_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x: current_x, y, width: word_width, height: line_height },
                        style: style.clone(),
                        children: vec![],
                        text_content: Some(word.to_string()),
                    };
                    
                    max_height = max_height.max(line_height);
                    current_x += word_width;
                    
                    // Add spacing after word
                    if current_x + space_width <= x + width {
                        current_x += space_width;
                    }
                    
                    line_boxes.push(word_box);
                }
            } else {
                // Non-text child - layout normally
                let mut child_box = self.layout_inline_element(dom, stylesheet, child_id, current_x, y, width - (current_x - x), exclude_tags);
                
                let child_width = child_box.dimensions.width;
                let child_height = child_box.dimensions.height;
                
                // Check if this child exceeds the available width
                let child_right = current_x + child_width;
                let available_width = x + width;
                
                if child_right > available_width && current_x > x {
                    // Child doesn't fit on current line - wrap to next line
                    total_height += max_height;
                    y = y + max_height;
                    current_x = x;
                    max_height = 0.0;
                    
                    // Update child position to new line
                    child_box.dimensions.x = current_x;
                    child_box.dimensions.y = y;
                } else {
                    // Child fits on current line
                    child_box.dimensions.x = current_x;
                    child_box.dimensions.y = y;
                }
                
                max_height = max_height.max(child_height);
                current_x += child_width;
                line_boxes.push(child_box);
            }
        }

        total_height += max_height;
        LayoutBox {
            node_id: 0,
            box_type: BoxType::Block,
            dimensions: Dimensions { x, y, width, height: total_height },
            style: Style::new(),
            children: line_boxes,
            text_content: None,
        }
    }

    fn layout_inline_element(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        node_id: NodeId,
        x: f32,
        y: f32,
        max_width: f32,
        exclude_tags: &[&str],
    ) -> LayoutBox {
        let style = stylesheet.compute_style(dom, node_id);

        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Text(text) => {
                // Split text into words and create a box for the entire text
                // The wrapping will be handled at the line layout level
                self.layout_text_run(node_id, x, y, text, &style)
            }
            crate::dom::NodeType::Element(el) => {
                if el.tag_name == "img" {
                    LayoutBox {
                        node_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x, y, width: 100.0, height: 80.0 },
                        style,
                        children: vec![],
                        text_content: None,
                    }
                } else {
                    let mut children_boxes = Vec::new();
                    let mut current_x = x;
                    let mut max_height = 16.0_f32;

                    for &child_id in &dom.nodes[node_id].children {
                        let should_exclude = if let crate::dom::NodeType::Element(el) = &dom.nodes[child_id].node_type {
                            exclude_tags.contains(&el.tag_name.as_str())
                        } else {
                            false
                        };

                        if !should_exclude {
                            let child_box = self.layout_inline_element(dom, stylesheet, child_id, current_x, y, max_width - (current_x - x), exclude_tags);
                            max_height = max_height.max(child_box.dimensions.height);
                            current_x += child_box.dimensions.width;
                            children_boxes.push(child_box);
                        }
                    }

                    LayoutBox {
                        node_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x, y, width: current_x - x, height: max_height },
                        style,
                        children: children_boxes,
                        text_content: None,
                    }
                }
            }
        }
    }

    fn layout_text_run(
        &self,
        node_id: NodeId,
        x: f32,
        y: f32,
        text: &str,
        style: &Style,
    ) -> LayoutBox {
        let font_size = style.get_font_size();
        let line_height = font_size * 1.2;
        
        // Calculate width based on text content
        // Use a heuristic: approximately 0.6 * font_size per character on average
        let text_width = text.len() as f32 * font_size * 0.6;

        LayoutBox {
            node_id,
            box_type: BoxType::Inline,
            dimensions: Dimensions { x, y, width: text_width, height: line_height },
            style: style.clone(),
            children: vec![],
            text_content: Some(text.to_string()),
        }
    }
}
