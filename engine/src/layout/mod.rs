use crate::dom::{Dom, NodeId};
use crate::style::{Stylesheet, Style};
use std::collections::HashMap;

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
    pub children: Vec<LayoutBox>,
}

pub struct LayoutEngine;

impl LayoutEngine {
    pub fn new() -> Self { Self }

    pub fn layout(&self, dom: &Dom, stylesheet: &Stylesheet) -> LayoutBox {
        // Root node
        let root_id = dom.root();
        self.layout_node(dom, stylesheet, root_id, 0.0, 0.0)
    }

    fn layout_node(&self, dom: &Dom, stylesheet: &Stylesheet, node_id: NodeId, x: f32, y: f32) -> LayoutBox {
        let node_style = stylesheet.compute_style(dom, node_id);
        let box_type = match &dom.nodes[node_id].node_type {
            crate::dom::NodeType::Text(_) => BoxType::Inline,
            crate::dom::NodeType::Element(el) => {
                match el.tag_name.as_str() {
                    "p" | "div" | "body" | "html" | "document" => BoxType::Block,
                    _ => BoxType::Inline,
                }
            }
        };

        // Simple sizing: width = 500 for block, 0 for inline text (we can improve later)
        let width = match box_type {
            BoxType::Block => 500.0,
            BoxType::Inline => 0.0,
        };

        let mut height = 0.0;
        let mut children_boxes = Vec::new();

        for &child_id in &dom.nodes[node_id].children {
            let child_box = self.layout_node(dom, stylesheet, child_id, x, y + height);
            height += child_box.dimensions.height;
            children_boxes.push(child_box);
        }

        // Text node height placeholder
        if let crate::dom::NodeType::Text(text) = &dom.nodes[node_id].node_type {
            height = 16.0; // fixed line height
        }

        LayoutBox {
            node_id,
            box_type,
            dimensions: Dimensions { x, y, width, height },
            children: children_boxes,
        }
    }
}
