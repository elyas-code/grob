// engine/src/layout/mod.rs
// Rearchitected layout engine with proper CSS Box Model support
//
// CSS Box Model:
// +------------------------------------------+
// |              MARGIN                      |
// |  +------------------------------------+  |
// |  |           BORDER (not impl yet)    |  |
// |  |  +------------------------------+  |  |
// |  |  |         PADDING              |  |  |
// |  |  |  +------------------------+  |  |  |
// |  |  |  |       CONTENT          |  |  |  |
// |  |  |  +------------------------+  |  |  |
// |  |  +------------------------------+  |  |
// |  +------------------------------------+  |
// +------------------------------------------+
//
// Key principle: CSS "width" property sets CONTENT width, not border-box width.

use crate::dom::{Dom, NodeId};
use crate::font::FontManager;
use crate::style::{Stylesheet, Style, Viewport};

pub const CSS_PX_SCALE: f32 = 1.0;
pub const BASE_FONT_SIZE: f32 = 16.0;

const DEBUG_LAYOUT: bool = false;

fn layout_log(msg: &str) {
    if DEBUG_LAYOUT {
        eprintln!("[LAYOUT] {}", msg);
    }
}

fn text_log(msg: &str) {
    if DEBUG_LAYOUT {
        eprintln!("[TEXT] {}", msg);
    }
}

fn get_tag_name(dom: &Dom, node_id: NodeId) -> String {
    match &dom.nodes[node_id].node_type {
        crate::dom::NodeType::Element(el) => el.tag_name.clone(),
        crate::dom::NodeType::Text(t) => format!("#text({})", &t[..t.len().min(20)]),
    }
}

#[derive(Debug, Clone)]
pub enum BoxType {
    Block,
    Inline,
}

#[derive(Debug, Clone)]
pub struct Dimensions {
    pub x: f32,       // Border-box x position
    pub y: f32,       // Border-box y position  
    pub width: f32,   // Border-box width (content + padding)
    pub height: f32,  // Border-box height (content + padding)
}

#[derive(Debug, Clone)]
pub struct LayoutBox {
    pub node_id: NodeId,
    pub box_type: BoxType,
    pub dimensions: Dimensions,
    pub style: Style,
    pub children: Vec<LayoutBox>,
    pub text_content: Option<String>,
}

pub struct LayoutEngine {
    viewport: Viewport,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutEngine {
    pub fn new() -> Self {
        Self {
            viewport: Viewport::default(),
        }
    }

    pub fn with_viewport(viewport: Viewport) -> Self {
        Self { viewport }
    }

    pub fn set_viewport(&mut self, viewport: Viewport) {
        self.viewport = viewport;
    }

    pub fn get_viewport(&self) -> Viewport {
        self.viewport
    }
    
    /// Measure text width using font manager (accurate)
    fn measure_text_width(&self, text: &str, font_manager: &mut FontManager, font_family: &str, font_size: f32, bold: bool, italic: bool) -> f32 {
        font_manager.measure_text(text, font_family, font_size, bold, italic)
    }

    pub fn layout(&self, dom: &Dom, stylesheet: &Stylesheet) -> LayoutBox {
        self.layout_with_viewport(dom, stylesheet, self.viewport.width)
    }

    pub fn layout_with_viewport(&self, dom: &Dom, stylesheet: &Stylesheet, viewport_width: f32) -> LayoutBox {
        let viewport = Viewport::new(viewport_width, self.viewport.height);
        let root_id = dom.root();
        let exclude_tags = ["head", "meta", "link", "title", "style", "script", "base", "noscript"];
        
        // Use a temporary font manager for fallback - this path doesn't use accurate text metrics
        let mut font_manager = FontManager::new();
        let mut root_box = self.layout_root_element(dom, stylesheet, root_id, &viewport, &exclude_tags, &mut font_manager);
        root_box.dimensions.width = viewport.width;
        root_box.dimensions.height = root_box.dimensions.height.max(viewport.height);
        root_box
    }

    /// Layout with font manager for accurate text measurement
    pub fn layout_with_full_viewport(&self, dom: &Dom, stylesheet: &Stylesheet, viewport: Viewport, font_manager: &mut FontManager) -> LayoutBox {
        layout_log(&format!("=== LAYOUT START === viewport: {}x{}", viewport.width, viewport.height));
        let root_id = dom.root();
        let exclude_tags = ["head", "meta", "link", "title", "style", "script", "base", "noscript"];
        
        let mut root_box = self.layout_root_element(dom, stylesheet, root_id, &viewport, &exclude_tags, font_manager);
        root_box.dimensions.width = viewport.width;
        root_box.dimensions.height = root_box.dimensions.height.max(viewport.height);
        layout_log(&format!("=== LAYOUT END === root box: x={}, y={}, w={}, h={}", 
            root_box.dimensions.x, root_box.dimensions.y, 
            root_box.dimensions.width, root_box.dimensions.height));
        root_box
    }

    fn is_root_element(&self, dom: &Dom, node_id: NodeId) -> bool {
        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Element(el) => matches!(el.tag_name.as_str(), "document" | "html" | "body"),
            _ => false,
        }
    }

    fn layout_root_element(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        node_id: NodeId,
        viewport: &Viewport,
        exclude_tags: &[&str],
        font_manager: &mut FontManager,
    ) -> LayoutBox {
        let tag = get_tag_name(dom, node_id);
        layout_log(&format!("layout_root_element: <{}> viewport_width={}", tag, viewport.width));
        
        let style = stylesheet.compute_style_with_viewport(dom, node_id, viewport);
        
        // Debug: log all style properties for this element
        layout_log(&format!("  <{}> style props: {:?}", tag, style.properties.keys().collect::<Vec<_>>()));
        if let Some(margin) = style.get("margin") {
            layout_log(&format!("  <{}> margin property: '{}'", tag, margin));
        }
        if let Some(width) = style.get("width") {
            layout_log(&format!("  <{}> width property: '{}'", tag, width));
        }
        
        // Get margin with viewport height awareness for vh units
        let (body_mt, body_mr, body_mb, body_ml) = style.get_margin_with_viewport(viewport.height);
        let has_auto_margin = style.has_auto_horizontal_margin();
        layout_log(&format!("  <{}> margins: top={}, right={}, bottom={}, left={}, auto={}", 
            tag, body_mt, body_mr, body_mb, body_ml, has_auto_margin));
        
        // Check for explicit width on body (e.g., width: 60vw)
        let explicit_width = style.get_width_px(viewport.width);
        layout_log(&format!("  <{}> explicit_width: {:?}", tag, explicit_width));
        
        // Calculate the actual content width for this root element
        let (content_x, content_width, box_x, box_width) = if let Some(w) = explicit_width {
            // Element has explicit width - center it with auto margins
            if has_auto_margin {
                let remaining = (viewport.width - w).max(0.0);
                let margin = remaining / 2.0;
                layout_log(&format!("  <{}> auto margin centering: width={}, remaining={}, margin={}", tag, w, remaining, margin));
                (margin, w, 0.0, viewport.width)
            } else {
                // Explicit width but not centered
                (body_ml, w, 0.0, viewport.width)
            }
        } else {
            // No explicit width - fill viewport
            (0.0, viewport.width, 0.0, viewport.width)
        };
        
        layout_log(&format!("  <{}> layout: content_x={}, content_width={}", tag, content_x, content_width));
        
        let mut children_boxes = Vec::new();
        let mut current_y = body_mt;
        let children = dom.nodes[node_id].children.clone();

        for child_id in children {
            let should_exclude = if let crate::dom::NodeType::Element(el) = &dom.nodes[child_id].node_type {
                exclude_tags.contains(&el.tag_name.as_str())
            } else {
                false
            };

            if should_exclude {
                continue;
            }

            if self.is_root_element(dom, child_id) {
                let mut child_box = self.layout_root_element(dom, stylesheet, child_id, viewport, exclude_tags, font_manager);
                child_box.dimensions.x = content_x;
                child_box.dimensions.y = current_y;
                child_box.dimensions.width = content_width;
                current_y += child_box.dimensions.height;
                children_boxes.push(child_box);
            } else if self.is_list_container(dom, child_id) {
                // List containers (ul, ol)
                let child_style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let (child_mt, _, child_mb, _) = child_style.get_margin_with_viewport(viewport.height);
                current_y += child_mt;
                
                let list_box = self.layout_list_container(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager,
                    0,
                );
                current_y += list_box.dimensions.height + child_mb;
                children_boxes.push(list_box);
            } else if self.is_block_element(dom, child_id) {
                // Get child margins first to properly position
                let child_style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let (child_mt, _, child_mb, _) = child_style.get_margin_with_viewport(viewport.height);
                
                // Add top margin before laying out child
                current_y += child_mt;
                
                let child_box = self.layout_block_element(
                    dom, stylesheet, child_id, 
                    content_x, current_y, content_width, 
                    exclude_tags, viewport, font_manager
                );
                
                // Move down by child's border-box height plus bottom margin
                current_y += child_box.dimensions.height + child_mb;
                children_boxes.push(child_box);
            } else {
                let inline_children = vec![child_id];
                let line_box = self.layout_inline_line(
                    dom, stylesheet, &inline_children, 
                    content_x, current_y, content_width, 
                    exclude_tags, viewport, font_manager
                );
                // Only add line box if it has content (non-zero height)
                if line_box.dimensions.height > 0.0 {
                    current_y += line_box.dimensions.height;
                    children_boxes.push(line_box);
                }
            }
        }

        // Add bottom margin to content height
        let total_height = (current_y + body_mb).max(viewport.height);

        LayoutBox {
            node_id,
            box_type: BoxType::Block,
            dimensions: Dimensions { 
                x: box_x, 
                y: 0.0, 
                width: box_width, 
                height: total_height,
            },
            style,
            children: children_boxes,
            text_content: None,
        }
    }

    fn is_block_element(&self, dom: &Dom, node_id: NodeId) -> bool {
        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Text(_) => false,
            crate::dom::NodeType::Element(el) => {
                // Check if display is explicitly set via style attribute
                // For now, use HTML default block/inline classification
                // Per HTML spec, these elements have display: block by default
                matches!(el.tag_name.to_lowercase().as_str(),
                    // Document structure
                    "html" | "body" | "document" | "head" | "title" | "meta" | "link" | 
                    "style" | "script" | "noscript" | "template" |
                    // Sections
                    "article" | "aside" | "footer" | "header" | "nav" | "section" | "main" |
                    // Grouping content
                    "p" | "div" | "blockquote" | "pre" | "hr" | "address" |
                    "figure" | "figcaption" | "center" |
                    // Headings
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "hgroup" |
                    // Lists
                    "ul" | "ol" | "li" | "dl" | "dt" | "dd" | "dir" | "menu" |
                    // Tables
                    "table" | "caption" | "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" | "col" | "colgroup" |
                    // Forms
                    "form" | "fieldset" | "legend" | "label" | "input" | "button" | 
                    "select" | "textarea" | "option" | "optgroup" | "datalist" | "output" |
                    // Embedded content (often block-like)
                    "img" | "iframe" | "video" | "audio" | "canvas" | "object" | "embed" |
                    "picture" | "source" | "track" |
                    // Interactive
                    "details" | "summary" | "dialog" |
                    // Deprecated but still used
                    "xmp" | "listing" | "plaintext" | "frameset" | "frame" | "noframes"
                )
            }
        }
    }
    
    /// Check if element is a list container (ul or ol)
    fn is_list_container(&self, dom: &Dom, node_id: NodeId) -> bool {
        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Element(el) => matches!(el.tag_name.as_str(), "ul" | "ol"),
            _ => false,
        }
    }
    
    /// Check if element is a list item
    fn is_list_item(&self, dom: &Dom, node_id: NodeId) -> bool {
        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Element(el) => el.tag_name == "li",
            _ => false,
        }
    }
    
    /// Get the list type for marker generation
    fn get_list_type(&self, dom: &Dom, node_id: NodeId) -> Option<&str> {
        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Element(el) => {
                match el.tag_name.as_str() {
                    "ul" => Some("ul"),
                    "ol" => Some("ol"),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Layout a block-level element using the CSS Box Model.
    ///
    /// Parameters:
    /// - x, y: Position where this element's margin box starts
    /// - containing_width: Width of the containing block (parent's content area width)
    ///
    /// The containing_width is used to:
    /// 1. Calculate percentage-based widths (e.g., width: 60vw uses viewport, but width: 50% would use this)
    /// 2. Calculate auto margins for centering
    fn layout_block_element(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        node_id: NodeId,
        x: f32,
        y: f32,
        containing_width: f32,
        exclude_tags: &[&str],
        viewport: &Viewport,
        font_manager: &mut FontManager,
    ) -> LayoutBox {
        let tag = get_tag_name(dom, node_id);
        let style = stylesheet.compute_style_with_viewport(dom, node_id, viewport);
        
        // Step 1: Get padding values
        let (padding_top, padding_right, padding_bottom, padding_left) = style.get_padding();
        
        // Step 2: Get margin values with viewport height awareness for vh units
        let (margin_top, margin_right, margin_bottom, margin_left) = style.get_margin_with_viewport(viewport.height);
        let has_auto_margin = style.has_auto_horizontal_margin();
        
        layout_log(&format!("layout_block: <{}> at ({}, {}) containing_width={}", tag, x, y, containing_width));
        layout_log(&format!("  margins: t={}, r={}, b={}, l={}, auto={}", margin_top, margin_right, margin_bottom, margin_left, has_auto_margin));
        layout_log(&format!("  padding: t={}, r={}, b={}, l={}", padding_top, padding_right, padding_bottom, padding_left));
        
        // Check for explicit width
        let explicit_width = style.get_width_percentage().map(|f| viewport.width * f)
            .or_else(|| style.get_width_px(viewport.width));
        layout_log(&format!("  explicit_width: {:?}", explicit_width));
        
        // Step 3: Calculate content width
        let content_width = if let Some(w) = explicit_width {
            w
        } else {
            // Block elements fill available width (containing_width - padding - margins)
            let horizontal_margin = if has_auto_margin { 0.0 } else { margin_left + margin_right };
            (containing_width - padding_left - padding_right - horizontal_margin).max(0.0)
        };
        
        layout_log(&format!("  content_width: {}", content_width));
        
        // Step 4: Calculate border-box width (content + padding)
        let border_box_width = content_width + padding_left + padding_right;
        
        // Step 5: Calculate horizontal margins
        let (final_margin_left, final_margin_right) = if has_auto_margin {
            // Auto margins: distribute remaining space equally for centering
            let remaining = (containing_width - border_box_width).max(0.0);
            layout_log(&format!("  AUTO MARGIN: remaining={}, each side={}", remaining, remaining / 2.0));
            (remaining / 2.0, remaining / 2.0)
        } else {
            (margin_left, margin_right)
        };
        
        layout_log(&format!("  final margins: left={}, right={}", final_margin_left, final_margin_right));
        
        // Step 6: Calculate border-box position
        let border_box_x = x + final_margin_left;
        let border_box_y = y;
        
        layout_log(&format!("  border_box: x={}, y={}, width={}", border_box_x, border_box_y, border_box_width));
        
        // Step 7: Calculate content area position (inside padding)
        let content_x = border_box_x + padding_left;
        let content_y = border_box_y + padding_top;
        
        // Step 8: Layout children within the content area
        let mut children_boxes = Vec::new();
        let mut current_y = content_y;
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

            // Check for list containers first (ul, ol)
            if self.is_list_container(dom, child_id) {
                let child_style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let (child_mt, _, child_mb, _) = child_style.get_margin_with_viewport(viewport.height);
                current_y += child_mt;
                
                let list_box = self.layout_list_container(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager,
                    0, // list depth starts at 0
                );
                current_y += list_box.dimensions.height + child_mb;
                children_boxes.push(list_box);
                child_idx += 1;
            } else if self.is_block_element(dom, child_id) {
                // Block element: layout within content area
                // Child's containing width is THIS element's content width
                // Get child margins first to properly position
                let child_style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let (child_mt, _, child_mb, _) = child_style.get_margin_with_viewport(viewport.height);
                
                // Add top margin before laying out child
                current_y += child_mt;
                
                let child_box = self.layout_block_element(
                    dom, stylesheet, child_id, 
                    content_x, current_y, content_width, 
                    exclude_tags, viewport, font_manager
                );
                
                // Move down by the child's border-box height plus bottom margin
                current_y += child_box.dimensions.height + child_mb;
                children_boxes.push(child_box);
                child_idx += 1;
            } else {
                // Inline or text - collect consecutive inline children
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

                // Layout inline children as a line box
                let line_box = self.layout_inline_line(
                    dom, stylesheet, &inline_children, 
                    content_x, current_y, content_width, 
                    exclude_tags, viewport, font_manager
                );
                // Only add line box if it has content (non-zero height)
                if line_box.dimensions.height > 0.0 {
                    current_y += line_box.dimensions.height;
                    children_boxes.push(line_box);
                }
            }
        }

        // Step 9: Calculate content height (determined by children)
        let content_height = (current_y - content_y).max(0.0);
        
        // Step 10: Calculate border-box height
        let border_box_height = content_height + padding_top + padding_bottom;
        
        // Step 11: Build the layout box
        // dimensions represents the border-box (what gets painted with background)
        LayoutBox {
            node_id,
            box_type: BoxType::Block,
            dimensions: Dimensions { 
                x: border_box_x,
                y: border_box_y, 
                width: border_box_width, 
                height: border_box_height,  // Don't force min height - empty blocks should be zero-height
            },
            style,
            children: children_boxes,
            text_content: None,
        }
    }

    /// Layout a list container (ul or ol) with proper indentation
    fn layout_list_container(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        node_id: NodeId,
        x: f32,
        y: f32,
        containing_width: f32,
        exclude_tags: &[&str],
        viewport: &Viewport,
        font_manager: &mut FontManager,
        list_depth: usize,
    ) -> LayoutBox {
        let tag = get_tag_name(dom, node_id);
        let style = stylesheet.compute_style_with_viewport(dom, node_id, viewport);
        let list_type = self.get_list_type(dom, node_id);
        
        layout_log(&format!("layout_list_container: <{}> depth={} at ({}, {})", tag, list_depth, x, y));
        
        // Default list indentation: 40px (standard browser default)
        let list_indent = 40.0;
        
        // Get any explicit padding from style, or use default
        let (padding_top, padding_right, padding_bottom, padding_left) = style.get_padding();
        let effective_padding_left = if padding_left > 0.0 { padding_left } else { list_indent };
        
        // Get margins (margin_top/bottom not currently used for list containers)
        let (_margin_top, margin_right, _margin_bottom, margin_left) = style.get_margin_with_viewport(viewport.height);
        
        // Calculate content area
        let border_box_x = x + margin_left;
        let border_box_y = y;
        let content_width = (containing_width - margin_left - margin_right - effective_padding_left - padding_right).max(0.0);
        let content_x = border_box_x + effective_padding_left;
        let content_y = border_box_y + padding_top;
        
        layout_log(&format!("  list indent: {}, content_x: {}, content_width: {}", effective_padding_left, content_x, content_width));
        
        // Layout children (list items)
        let mut children_boxes = Vec::new();
        let mut current_y = content_y;
        let children = dom.nodes[node_id].children.clone();
        let mut item_index = 0;

        for child_id in children {
            let child_tag = get_tag_name(dom, child_id);
            layout_log(&format!("  list_container child: <{}> is_list_item={} is_list_container={} is_block={}",
                child_tag,
                self.is_list_item(dom, child_id),
                self.is_list_container(dom, child_id),
                self.is_block_element(dom, child_id)));
            
            let should_exclude = if let crate::dom::NodeType::Element(el) = &dom.nodes[child_id].node_type {
                exclude_tags.contains(&el.tag_name.as_str())
            } else {
                false
            };

            if should_exclude {
                continue;
            }

            if self.is_list_item(dom, child_id) {
                item_index += 1;
                layout_log(&format!("    -> calling layout_list_item for <{}> #{}", child_tag, item_index));
                let li_box = self.layout_list_item(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager,
                    list_type, item_index, list_depth,
                );
                current_y += li_box.dimensions.height;
                children_boxes.push(li_box);
            } else if self.is_list_container(dom, child_id) {
                // Nested list
                let nested_list = self.layout_list_container(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager,
                    list_depth + 1,
                );
                current_y += nested_list.dimensions.height;
                children_boxes.push(nested_list);
            } else if self.is_block_element(dom, child_id) {
                // Other block element inside list (unusual but possible)
                let child_style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let (child_mt, _, child_mb, _) = child_style.get_margin_with_viewport(viewport.height);
                current_y += child_mt;
                
                let child_box = self.layout_block_element(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager
                );
                current_y += child_box.dimensions.height + child_mb;
                children_boxes.push(child_box);
            }
            // Skip non-li inline content in lists (whitespace text nodes, etc.)
        }

        let content_height = (current_y - content_y).max(0.0);
        let border_box_height = content_height + padding_top + padding_bottom;
        let border_box_width = content_width + effective_padding_left + padding_right;

        LayoutBox {
            node_id,
            box_type: BoxType::Block,
            dimensions: Dimensions {
                x: border_box_x,
                y: border_box_y,
                width: border_box_width,
                height: border_box_height,
            },
            style,
            children: children_boxes,
            text_content: None,
        }
    }

    /// Layout a list item (li) with marker
    fn layout_list_item(
        &self,
        dom: &Dom,
        stylesheet: &Stylesheet,
        node_id: NodeId,
        x: f32,
        y: f32,
        containing_width: f32,
        exclude_tags: &[&str],
        viewport: &Viewport,
        font_manager: &mut FontManager,
        list_type: Option<&str>,
        item_index: usize,
        _list_depth: usize,
    ) -> LayoutBox {
        let style = stylesheet.compute_style_with_viewport(dom, node_id, viewport);
        let font_size = style.get_font_size();
        let font_family = style.get_font_family();
        let is_bold = style.is_bold();
        let is_italic = style.is_italic();
        let line_height = font_size * 1.2;
        
        // Generate marker text
        let marker_text = match list_type {
            Some("ul") => "•".to_string(),
            Some("ol") => format!("{}.", item_index),
            _ => "•".to_string(),
        };
        
        // Measure marker width
        let marker_width = self.measure_text_width(&marker_text, font_manager, &font_family, font_size, is_bold, is_italic);
        let marker_spacing = font_size * 0.5; // Space between marker and content
        let marker_area_width = marker_width + marker_spacing;
        
        layout_log(&format!("layout_list_item: item #{} marker='{}' marker_width={:.2}", item_index, marker_text, marker_width));
        
        // Position marker to the left of content area (outside)
        let marker_x = x - marker_area_width;
        
        // Content starts at x (marker is outside/before)
        let content_x = x;
        let content_y = y;
        let content_width = containing_width;
        
        // Create marker box
        let marker_box = LayoutBox {
            node_id,
            box_type: BoxType::Inline,
            dimensions: Dimensions {
                x: marker_x.max(0.0),
                y: content_y,
                width: marker_width,
                height: line_height,
            },
            style: style.clone(),
            children: vec![],
            text_content: Some(marker_text),
        };
        
        // Layout content (children of li)
        let mut children_boxes = vec![marker_box];
        let mut current_y = content_y;
        let children = dom.nodes[node_id].children.clone();
        let mut child_idx = 0;
        let mut first_line = true;

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

            if self.is_list_container(dom, child_id) {
                // Nested list inside li
                let nested_list = self.layout_list_container(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager,
                    _list_depth + 1,
                );
                current_y += nested_list.dimensions.height;
                children_boxes.push(nested_list);
                child_idx += 1;
                first_line = false;
            } else if self.is_block_element(dom, child_id) {
                // Block element inside li
                let child_style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let (child_mt, _, child_mb, _) = child_style.get_margin_with_viewport(viewport.height);
                current_y += child_mt;
                
                let child_box = self.layout_block_element(
                    dom, stylesheet, child_id,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager
                );
                current_y += child_box.dimensions.height + child_mb;
                children_boxes.push(child_box);
                child_idx += 1;
                first_line = false;
            } else {
                // Inline or text content
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
                    
                    if self.is_block_element(dom, next_id) || self.is_list_container(dom, next_id) {
                        break;
                    }
                    inline_children.push(next_id);
                    child_idx += 1;
                }

                let line_box = self.layout_inline_line(
                    dom, stylesheet, &inline_children,
                    content_x, current_y, content_width,
                    exclude_tags, viewport, font_manager
                );
                
                if line_box.dimensions.height > 0.0 {
                    // Align first line with marker
                    if first_line {
                        first_line = false;
                    }
                    current_y += line_box.dimensions.height;
                    children_boxes.push(line_box);
                }
            }
        }

        let total_height = (current_y - content_y).max(line_height);

        LayoutBox {
            node_id,
            box_type: BoxType::Block,
            dimensions: Dimensions {
                x: content_x,
                y: content_y,
                width: content_width,
                height: total_height,
            },
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
        viewport: &Viewport,
        font_manager: &mut FontManager,
    ) -> LayoutBox {
        let mut line_boxes = Vec::new();
        let mut current_x = x;
        let mut max_height = 0.0_f32;
        let mut total_height = 0.0_f32;
        let start_y = y;

        text_log(&format!("=== layout_inline_line: x={}, y={}, width={} ===", x, y, width));

        for &child_id in inline_children {
            if let crate::dom::NodeType::Text(text) = &dom.nodes[child_id].node_type {
                // Skip whitespace-only text nodes
                if text.trim().is_empty() {
                    text_log(&format!("  SKIP whitespace-only text node"));
                    continue;
                }
                
                let style = stylesheet.compute_style_with_viewport(dom, child_id, viewport);
                let font_size = style.get_font_size();
                let font_family = style.get_font_family();
                let is_bold = style.is_bold();
                let is_italic = style.is_italic();
                let line_height = font_size * 1.2;
                
                text_log(&format!("  text node: '{}' font_size={}", text.chars().take(50).collect::<String>(), font_size));
                
                let words: Vec<&str> = text.split_whitespace().collect();
                if words.is_empty() {
                    text_log(&format!("  SKIP empty words after split"));
                    continue;
                }
                text_log(&format!("  split into {} words: {:?}", words.len(), words));
                
                // Measure space width using actual font
                let space_width = self.measure_text_width(" ", font_manager, font_family, font_size, is_bold, is_italic);
                
                for (word_idx, word) in words.iter().enumerate() {
                    // Measure word using actual font metrics
                    let word_width = self.measure_text_width(word, font_manager, font_family, font_size, is_bold, is_italic);
                    
                    text_log(&format!("    word[{}] '{}': width={:.2}, space_width={:.2}, current_x={:.2}, available={:.2}", 
                        word_idx, word, word_width, space_width, current_x, x + width - current_x));
                    
                    // Check if word fits on current line
                    if current_x + word_width > x + width && current_x > x {
                        text_log(&format!("      -> WRAP: word doesn't fit (needs {:.2}, have {:.2})", word_width, x + width - current_x));
                        // Word doesn't fit, check if we need character-level wrapping
                        if word_width > width {
                            text_log(&format!("      -> CHARACTER WRAP: word wider than line ({:.2} > {:.2})", word_width, width));
                            // Word is wider than available width, do character wrapping
                            let mut remaining_word = *word;
                            while !remaining_word.is_empty() {
                                let mut char_count = 0;
                                let mut accumulated_width = 0.0;
                                let available = if current_x > x { x + width - current_x } else { width };
                                
                                for c in remaining_word.chars() {
                                    // Measure character using actual font
                                    let char_str = c.to_string();
                                    let char_width = self.measure_text_width(&char_str, font_manager, font_family, font_size, is_bold, is_italic);
                                    if accumulated_width + char_width > available && char_count > 0 {
                                        break;
                                    }
                                    accumulated_width += char_width;
                                    char_count += 1;
                                }
                                
                                if char_count == 0 {
                                    // Need new line first
                                    total_height += max_height;
                                    y += max_height;
                                    current_x = x;
                                    max_height = 0.0;
                                    continue;
                                }
                                
                                let (chunk, rest) = remaining_word.split_at(
                                    remaining_word.char_indices()
                                        .nth(char_count)
                                        .map(|(i, _)| i)
                                        .unwrap_or(remaining_word.len())
                                );
                                remaining_word = rest;
                                
                                // Measure chunk using actual font
                                let chunk_width = self.measure_text_width(chunk, font_manager, font_family, font_size, is_bold, is_italic);
                                
                                let word_box = LayoutBox {
                                    node_id: child_id,
                                    box_type: BoxType::Inline,
                                    dimensions: Dimensions { x: current_x, y, width: chunk_width, height: line_height },
                                    style: style.clone(),
                                    children: vec![],
                                    text_content: Some(chunk.to_string()),
                                };
                                
                                max_height = max_height.max(line_height);
                                current_x += chunk_width;
                                line_boxes.push(word_box);
                                
                                if !remaining_word.is_empty() {
                                    total_height += max_height;
                                    y += max_height;
                                    current_x = x;
                                    max_height = 0.0;
                                }
                            }
                            continue;
                        } else {
                            // Normal line break
                            text_log(&format!("      -> LINE BREAK"));
                            total_height += max_height;
                            y += max_height;
                            current_x = x;
                            max_height = 0.0;
                        }
                    }
                    
                    text_log(&format!("      -> PLACE at x={:.2}, word_width={:.2}", current_x, word_width));
                    
                    let word_box = LayoutBox {
                        node_id: child_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x: current_x, y, width: word_width, height: line_height },
                        style: style.clone(),
                        children: vec![],
                        text_content: Some(word.to_string()),
                    };
                    
                    max_height = max_height.max(line_height);
                    let _old_x = current_x;
                    current_x += word_width;
                    
                    // Add space after word (except at line end)
                    let is_last_word = word_idx == words.len() - 1;
                    if !is_last_word && current_x + space_width <= x + width {
                        text_log(&format!("      -> ADD SPACE: {:.2} (current_x: {:.2} -> {:.2})", space_width, current_x, current_x + space_width));
                        current_x += space_width;
                    } else if !is_last_word {
                        text_log(&format!("      -> NO SPACE (would overflow): space={:.2}, available={:.2}", space_width, x + width - current_x));
                    }
                    
                    line_boxes.push(word_box);
                }
            } else {
                let mut child_box = self.layout_inline_element(dom, stylesheet, child_id, current_x, y, width - (current_x - x), exclude_tags, viewport, font_manager);
                
                let child_width = child_box.dimensions.width;
                let child_height = child_box.dimensions.height;
                
                // Skip inline elements with zero dimensions
                if child_width <= 0.0 && child_height <= 0.0 {
                    continue;
                }
                
                if current_x + child_width > x + width && current_x > x {
                    total_height += max_height;
                    y += max_height;
                    current_x = x;
                    max_height = 0.0;
                    child_box.dimensions.x = current_x;
                    child_box.dimensions.y = y;
                }
                
                max_height = max_height.max(child_height);
                current_x += child_width;
                line_boxes.push(child_box);
            }
        }

        total_height += max_height;
        
        // Filter out any boxes with zero dimensions
        let visible_boxes: Vec<_> = line_boxes.into_iter()
            .filter(|b| b.dimensions.width > 0.0 || b.dimensions.height > 0.0)
            .collect();
        
        // Only create line box if we have visible content
        if visible_boxes.is_empty() || total_height <= 0.0 {
            layout_log(&format!("  inline_line: no visible content, returning empty box"));
            // Return a zero-height box
            return LayoutBox {
                node_id: 0,
                box_type: BoxType::Block,
                dimensions: Dimensions { x, y: start_y, width: 0.0, height: 0.0 },
                style: Style::new(),
                children: vec![],
                text_content: None,
            };
        }
        
        layout_log(&format!("  inline_line: {} children, height={}", visible_boxes.len(), total_height));
        LayoutBox {
            node_id: 0,
            box_type: BoxType::Block,
            dimensions: Dimensions { x, y: start_y, width, height: total_height },
            style: Style::new(),
            children: visible_boxes,
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
        viewport: &Viewport,
        font_manager: &mut FontManager,
    ) -> LayoutBox {
        let style = stylesheet.compute_style_with_viewport(dom, node_id, viewport);

        match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Text(text) => {
                // Skip whitespace-only text nodes
                if text.trim().is_empty() {
                    return LayoutBox {
                        node_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x, y, width: 0.0, height: 0.0 },
                        style: style.clone(),
                        children: vec![],
                        text_content: None,
                    };
                }
                
                let font_size = style.get_font_size();
                let line_height = font_size * 1.2;
                let font_family = style.get_font_family();
                let is_bold = style.get_font_weight() == "bold";
                let is_italic = style.get_font_style() == "italic";
                let text_width = self.measure_text_width(text, font_manager, &font_family, font_size, is_bold, is_italic);

                LayoutBox {
                    node_id,
                    box_type: BoxType::Inline,
                    dimensions: Dimensions { x, y, width: text_width.min(max_width), height: line_height },
                    style: style.clone(),
                    children: vec![],
                    text_content: Some(text.to_string()),
                }
            }
            crate::dom::NodeType::Element(el) => {
                if el.tag_name == "img" {
                    LayoutBox {
                        node_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x, y, width: 100.0_f32.min(max_width), height: 80.0 },
                        style,
                        children: vec![],
                        text_content: None,
                    }
                } else {
                    let mut children_boxes = Vec::new();
                    let mut current_x = x;
                    let mut max_height = 0.0_f32; // Start with 0 height, don't assume 16px

                    for &child_id in &dom.nodes[node_id].children {
                        let should_exclude = if let crate::dom::NodeType::Element(el) = &dom.nodes[child_id].node_type {
                            exclude_tags.contains(&el.tag_name.as_str())
                        } else {
                            false
                        };

                        if !should_exclude {
                            let remaining_width = (x + max_width - current_x).max(0.0);
                            let child_box = self.layout_inline_element(dom, stylesheet, child_id, current_x, y, remaining_width, exclude_tags, viewport, font_manager);
                            // Only count child if it has content
                            if child_box.dimensions.width > 0.0 || child_box.dimensions.height > 0.0 {
                                max_height = max_height.max(child_box.dimensions.height);
                                current_x += child_box.dimensions.width;
                                children_boxes.push(child_box);
                            }
                        }
                    }

                    LayoutBox {
                        node_id,
                        box_type: BoxType::Inline,
                        dimensions: Dimensions { x, y, width: (current_x - x).min(max_width), height: max_height },
                        style,
                        children: children_boxes,
                        text_content: None,
                    }
                }
            }
        }
    }
}
