use crate::dom::NodeId;
use crate::dom::{Dom, NodeType};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Style {
    pub properties: HashMap<String, String>,
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
            for rule in &self.rules {
                match &rule.selector {
                    Selector::Tag(tag) if tag == &el.tag_name => result.properties.extend(rule.declarations.properties.clone()),
                    Selector::Id(id) => {
                        if el.attributes.iter().any(|(k,vv)| k=="id" && vv==id) {
                            result.properties.extend(rule.declarations.properties.clone());
                        }
                    },
                    Selector::Class(class) => {
                        if el.attributes.iter().any(|(k,vv)| k=="class" && vv==class) {
                            result.properties.extend(rule.declarations.properties.clone());
                        }
                    },
                    _ => {}
                }
            }
        }

        result
    }
}