use crate::dom::{Dom, NodeId};
use super::tokenizer::{Token, Tokenizer};

#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    InTable,
    InSelect,
    InForeignContent,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
}

pub struct HtmlParser {
    tokenizer: Tokenizer,
}

// Auto-closing tags that force parent closure
const AUTO_CLOSING_TAGS: &[&str] = &[
    "p", "li", "dd", "dt", "option", "optgroup", "tr", "td", "th", 
];

// Tags that can be implicitly closed (not currently used, but may be useful for future HTML5 spec compliance)
#[allow(dead_code)]
const FORMATTING_TAGS: &[&str] = &[
    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt", "u",
];

impl HtmlParser {
    pub fn new(input: &str) -> Self {
        Self {
            tokenizer: Tokenizer::new(input),
        }
    }

    pub fn parse(mut self) -> Dom {
        let mut dom = Dom::new();
        let document = dom.create_element("document", vec![], None);
        let mut stack: Vec<NodeId> = vec![document];
        let mut mode = InsertionMode::Initial;
        let _fragment_context: Option<String> = None;

        while let Some(token) = self.tokenizer.next_token() {
            match &token {
                Token::Eof => break,
                Token::Comment(_) => {
                    // Append comment to current node (optional for now)
                    continue;
                }
                Token::Doctype { .. } => {
                    // Doctype only relevant in initial mode
                    if mode == InsertionMode::Initial {
                        mode = InsertionMode::BeforeHtml;
                    }
                }
                Token::StartTag { name, attributes, self_closing } => {
                    let tag = name.to_lowercase();

                    // -------- INITIAL MODE --------
                    if mode == InsertionMode::Initial {
                        // Auto-insert html element
                        let html = dom.create_element("html", vec![], Some(document));
                        stack.push(html);
                        mode = InsertionMode::BeforeHtml;
                    }

                    // -------- BEFORE HTML --------
                    if mode == InsertionMode::BeforeHtml {
                        if tag == "html" {
                            stack.pop(); // remove implicit html
                            let html = dom.create_element("html", attributes.clone(), Some(document));
                            stack.push(html);
                            mode = InsertionMode::BeforeHead;
                            continue;
                        } else {
                            // Auto-insert html
                            let html = dom.create_element("html", vec![], Some(document));
                            stack.pop();
                            stack.push(html);
                            mode = InsertionMode::BeforeHead;
                        }
                    }

                    // -------- BEFORE HEAD / IN HEAD --------
                    if mode == InsertionMode::BeforeHead || mode == InsertionMode::InHead {
                        if tag == "head" && mode == InsertionMode::BeforeHead {
                            if let Some(&parent) = stack.last() {
                                let head = dom.create_element("head", attributes.clone(), Some(parent));
                                stack.push(head);
                                mode = InsertionMode::InHead;
                            }
                            continue;
                        }
                        
                        if matches!(tag.as_str(), "meta" | "link" | "title" | "style" | "base") {
                            if mode == InsertionMode::BeforeHead {
                                if let Some(&parent) = stack.last() {
                                    let head = dom.create_element("head", vec![], Some(parent));
                                    stack.push(head);
                                    mode = InsertionMode::InHead;
                                }
                            }
                            if let Some(&parent) = stack.last() {
                                let elem_id = dom.create_element(&tag, attributes.clone(), Some(parent));
                                // Push non-self-closing elements to stack
                                if !*self_closing && !matches!(tag.as_str(), "meta" | "link") {
                                    stack.push(elem_id);
                                }
                            }
                            if *self_closing || matches!(tag.as_str(), "meta" | "link") {
                                // self-closing, don't push to stack
                            }
                            continue;
                        }

                        if mode == InsertionMode::InHead && tag != "head" {
                            stack.pop();
                            mode = InsertionMode::AfterHead;
                        }
                    }

                    // -------- AFTER HEAD / IN BODY --------
                    if mode == InsertionMode::AfterHead || mode == InsertionMode::InBody {
                        if mode == InsertionMode::AfterHead {
                            if let Some(&parent) = stack.last() {
                                let body = dom.create_element("body", vec![], Some(parent));
                                stack.push(body);
                                mode = InsertionMode::InBody;
                            }
                            // After transitioning to InBody, if this tag is body, skip creating another one
                            if tag == "body" {
                                continue;
                            }
                        }

                        // Handle auto-closing tags (like <p>, <li>, etc.)
                        if AUTO_CLOSING_TAGS.contains(&tag.as_str()) {
                            // Close any open tags of the same type by popping them
                            while let Some(&last) = stack.last() {
                                if last == document {
                                    break;
                                }
                                if let crate::dom::NodeType::Element(el) = &dom.nodes[last].node_type {
                                    if el.tag_name.to_lowercase() == tag {
                                        // Found open tag of same type, close it and stop
                                        stack.pop();
                                        break;
                                    }
                                    // Don't pop past body
                                    if matches!(el.tag_name.as_str(), "body" | "html" | "document") {
                                        break;
                                    }
                                }
                                stack.pop();
                            }
                        }

                        if let Some(&parent) = stack.last() {
                            let id = dom.create_element(&tag, attributes.clone(), Some(parent));
                            
                            if !*self_closing && !matches!(tag.as_str(), "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr") {
                                stack.push(id);
                            }
                        }
                    }
                }

                Token::EndTag { name } => {
                    let tag = name.to_lowercase();

                    // Special handling for head-related elements
                    if mode == InsertionMode::InHead {
                        if tag == "head" {
                            stack.pop(); // Pop head element
                            mode = InsertionMode::AfterHead;
                            continue;
                        } else if matches!(tag.as_str(), "title" | "meta" | "link" | "style" | "base") {
                            // Pop the element if it matches
                            if let Some(&last) = stack.last() {
                                if let crate::dom::NodeType::Element(el) = &dom.nodes[last].node_type {
                                    if el.tag_name.to_lowercase() == tag {
                                        stack.pop();
                                    }
                                }
                            }
                            continue;
                        } else {
                            // Unrecognized tag in head mode - exit head mode
                            stack.pop(); // Pop head
                            mode = InsertionMode::AfterHead;
                            // Reprocess this end tag in AfterHead mode
                        }
                    }

                    // Before closing an element, auto-close any open auto-closing tags (like <p>)
                    if mode == InsertionMode::InBody && !matches!(tag.as_str(), "p" | "li" | "dd" | "dt" | "option" | "optgroup" | "tr" | "td" | "th") {
                        // We're closing a non-auto-closing tag, so close any open auto-closing tags first
                        while let Some(&last) = stack.last() {
                            if last == document {
                                break;
                            }
                            if let crate::dom::NodeType::Element(el) = &dom.nodes[last].node_type {
                                if matches!(el.tag_name.as_str(), "p" | "li" | "dd" | "dt" | "option" | "optgroup" | "tr" | "td" | "th") {
                                    // Found an auto-closing tag - check if we should close it
                                    if el.tag_name.to_lowercase() == tag {
                                        // This is the tag we're trying to close, stop here
                                        break;
                                    }
                                    // This is an auto-closing tag that's in the way - close it
                                    stack.pop();
                                } else {
                                    // Not an auto-closing tag
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    // Close elements until we find matching opening tag
                    while let Some(&last) = stack.last() {
                        if last == document {
                            break;
                        }

                        if let crate::dom::NodeType::Element(el) = &dom.nodes[last].node_type {
                            if el.tag_name.to_lowercase() == tag {
                                stack.pop();
                                break;
                            }

                            // Mismatch: don't pop scope-limiting elements
                            if matches!(el.tag_name.as_str(), "body" | "html" | "document") {
                                // Can't close past body/html/document
                                break;
                            }

                            // For other mismatches, pop the element
                            stack.pop();
                        } else {
                            stack.pop();
                        }
                    }

                    // Determine next mode based on current element
                    if let Some(&current) = stack.last() {
                        if let crate::dom::NodeType::Element(el) = &dom.nodes[current].node_type {
                            if el.tag_name == "body" {
                                mode = InsertionMode::InBody;
                            } else if el.tag_name == "head" {
                                mode = InsertionMode::InHead;
                            } else if el.tag_name == "html" {
                                mode = InsertionMode::AfterHead;
                            }
                        }
                    }
                }

                Token::Text(text) => {
                    // Skip whitespace-only text in certain modes
                    if text.trim().is_empty() && mode == InsertionMode::Initial {
                        continue;
                    }

                    if mode == InsertionMode::InBody || mode == InsertionMode::AfterHead || mode == InsertionMode::InHead {
                        if let Some(&parent) = stack.last() {
                            // Auto-close <p> before text if needed
                            if mode == InsertionMode::AfterHead {
                                let body = dom.create_element("body", vec![], Some(parent));
                                stack.push(body);
                                mode = InsertionMode::InBody;
                            }
                            dom.create_text(text, Some(parent));
                        }
                    }
                }
            }
        }

        dom
    }
}
