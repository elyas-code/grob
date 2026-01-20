use crate::dom::{Dom, NodeId};
use super::tokenizer::{Token, Tokenizer};

pub struct HtmlParser {
    tokenizer: Tokenizer,
}

impl HtmlParser {
    pub fn new(input: &str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
        }
    }

    pub fn parse(mut self) -> Dom {
        let mut dom = Dom::new();

        // 1. Always create the document root
        let root = dom.create_element("document", vec![], None);

        // 2. Stack starts with root
        let mut stack: Vec<NodeId> = vec![root];

        // Track if html/body tags were added
        let mut html_id: Option<NodeId> = None;
        let mut body_id: Option<NodeId> = None;

        while let Some(token) = self.tokenizer.next_token() {
            match token {
                Token::StartTag { name, attributes } => {
                    // Implicit <html> insertion
                    if html_id.is_none() && name.to_lowercase() != "html" {
                        html_id = Some(dom.create_element("html", vec![], Some(root)));
                        stack.push(html_id.unwrap());
                    }

                    // Implicit <body> insertion
                    if body_id.is_none() && name.to_lowercase() != "html" && name.to_lowercase() != "body" {
                        let parent = html_id.unwrap_or_else(|| stack.last().copied().unwrap());
                        body_id = Some(dom.create_element("body", vec![], Some(parent)));
                        stack.push(body_id.unwrap());
                    }

                    // Now add the actual start tag
                    let parent = stack.last().copied().unwrap();
                    let id = dom.create_element(&name, attributes, Some(parent));
                    stack.push(id);

                    // Track html/body explicitly
                    if name.to_lowercase() == "html" {
                        html_id = Some(stack.last().copied().unwrap());
                    }
                    if name.to_lowercase() == "body" {
                        body_id = Some(stack.last().copied().unwrap());
                    }
                }

                Token::EndTag { name } => {
                    // Pop until we find a matching tag or root
                    while let Some(&last) = stack.last() {
                        let node = &dom.nodes[last];
                        match &node.node_type {
                            crate::dom::NodeType::Element(el) if el.tag_name.to_lowercase() == name.to_lowercase() => {
                                stack.pop();
                                break;
                            }
                            _ => {
                                // Prevent popping the root
                                if last == root { break; }
                                stack.pop();
                            }
                        }
                    }
                }

                Token::Text(text) => {
                    if !text.trim().is_empty() {
                        // Insert text into <body> if it exists, else last node
                        let parent = body_id.or_else(|| stack.last().copied()).unwrap();
                        dom.create_text(&text, Some(parent));
                    }
                }
            }
        }

        dom
    }
}
