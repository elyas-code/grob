// HTML Tree Builder implementation following WHATWG HTML Living Standard
// Spec Reference: https://html.spec.whatwg.org/multipage/parsing.html#tree-construction
//
// IMPLEMENTATION STATUS:
// ✅ Initial mode - basic
// ✅ BeforeHtml mode - basic
// ✅ BeforeHead mode - basic
// ✅ InHead mode - partial (meta, link, title, style, base)
// ✅ AfterHead mode - basic
// ✅ InBody mode - basic element creation
// ⚠️ Text mode - partial
// ❌ InTable mode - not implemented
// ❌ InSelect mode - not implemented
// ❌ InForeignContent mode - not implemented
// ⚠️ AfterBody mode - partial
// ❌ InFrameset mode - not implemented
// ❌ AfterFrameset mode - not implemented
// ❌ AfterAfterBody mode - not implemented
//
// TODO(spec 13.2.6): Implement full adoption agency algorithm
// TODO(spec 13.2.6): Implement foster parenting
// TODO(spec 13.2.6): Implement AAA (adoption agency algorithm)

use crate::dom::{Dom, NodeId};
use super::tokenizer::{Token, Tokenizer, VOID_ELEMENTS};

/// Debug logging for tree construction
const DEBUG_TREE_BUILDER: bool = false;

fn tree_builder_log(msg: &str) {
    if DEBUG_TREE_BUILDER {
        eprintln!("[TREE_BUILDER] {}", msg);
    }
}

/// Insertion modes per spec 13.2.6
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    InHeadNoscript,
    AfterHead,
    InBody,
    Text,
    InTable,
    InTableText,
    InCaption,
    InColumnGroup,
    InTableBody,
    InRow,
    InCell,
    InSelect,
    InSelectInTable,
    InTemplate,
    AfterBody,
    InFrameset,
    AfterFrameset,
    AfterAfterBody,
    AfterAfterFrameset,
}

pub struct HtmlParser {
    tokenizer: Tokenizer,
    /// Buffer for accumulating character tokens into text nodes
    pending_text: String,
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
            pending_text: String::new(),
        }
    }

    /// Flush any pending text to the DOM
    /// Only creates a text node if there's meaningful content (not just whitespace)
    fn flush_pending_text(&mut self, dom: &mut Dom, parent: NodeId) {
        if !self.pending_text.is_empty() {
            // Only create text node if it has non-whitespace content
            // OR if it's meaningful whitespace (single space between inline elements)
            let trimmed = self.pending_text.trim();
            if !trimmed.is_empty() {
                tree_builder_log(&format!("Flushing text: {:?}", self.pending_text));
                dom.create_text(&self.pending_text, Some(parent));
            } else {
                tree_builder_log(&format!("Skipping whitespace-only text: {:?}", self.pending_text));
            }
            self.pending_text.clear();
        }
    }

    /// Convert attribute list from tokenizer format to DOM format
    fn convert_attributes(attributes: &[super::tokenizer::Attribute]) -> Vec<(String, String)> {
        attributes.iter().map(|a| (a.name.clone(), a.value.clone())).collect()
    }

    pub fn parse(mut self) -> Dom {
        let mut dom = Dom::new();
        let document = dom.create_element("document", vec![], None);
        let mut stack: Vec<NodeId> = vec![document];
        let mut mode = InsertionMode::Initial;
        let _fragment_context: Option<String> = None;

        while let Some(token) = self.tokenizer.next_token() {
            tree_builder_log(&format!("Mode: {:?}, Token: {:?}", mode, token));
            
            match &token {
                Token::Eof => {
                    // Flush any remaining text
                    if let Some(&parent) = stack.last() {
                        self.flush_pending_text(&mut dom, parent);
                    }
                    break;
                }
                Token::Comment(_) => {
                    // Flush text before comment
                    if let Some(&parent) = stack.last() {
                        self.flush_pending_text(&mut dom, parent);
                    }
                    // Append comment to current node (optional for now)
                    continue;
                }
                Token::Doctype { .. } => {
                    // Doctype only relevant in initial mode
                    if mode == InsertionMode::Initial {
                        mode = InsertionMode::BeforeHtml;
                    }
                }
                Token::Character(c) => {
                    // Accumulate characters into pending_text
                    self.pending_text.push(*c);
                    continue;
                }
                Token::StartTag { name, attributes, self_closing } => {
                    // Flush pending text before processing tag
                    if let Some(&parent) = stack.last() {
                        self.flush_pending_text(&mut dom, parent);
                    }
                    
                    let tag = name.to_lowercase();
                    let attrs = Self::convert_attributes(attributes);

                    // -------- INITIAL MODE --------
                    if mode == InsertionMode::Initial {
                        // Move directly to BeforeHtml without creating an element yet
                        mode = InsertionMode::BeforeHtml;
                    }

                    // -------- BEFORE HTML --------
                    if mode == InsertionMode::BeforeHtml {
                        if tag == "html" {
                            let html = dom.create_element("html", attrs.clone(), Some(document));
                            stack.push(html);
                            mode = InsertionMode::BeforeHead;
                            continue;
                        } else {
                            // Auto-insert html element
                            let html = dom.create_element("html", vec![], Some(document));
                            stack.push(html);
                            mode = InsertionMode::BeforeHead;
                            // Fall through to process this tag in BeforeHead mode
                        }
                    }

                    // -------- BEFORE HEAD / IN HEAD --------
                    if mode == InsertionMode::BeforeHead || mode == InsertionMode::InHead {
                        if tag == "head" && mode == InsertionMode::BeforeHead {
                            if let Some(&parent) = stack.last() {
                                let head = dom.create_element("head", attrs.clone(), Some(parent));
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
                                let elem_id = dom.create_element(&tag, attrs.clone(), Some(parent));
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
                                    // Don't pop past body, html, document
                                    if matches!(el.tag_name.as_str(), "body" | "html" | "document") {
                                        break;
                                    }
                                    // Don't pop past list containers when handling li
                                    if tag == "li" && matches!(el.tag_name.as_str(), "ul" | "ol") {
                                        break;
                                    }
                                    // Don't pop past definition lists when handling dt/dd
                                    if matches!(tag.as_str(), "dt" | "dd") && el.tag_name == "dl" {
                                        break;
                                    }
                                }
                                stack.pop();
                            }
                        }

                        if let Some(&parent) = stack.last() {
                            let id = dom.create_element(&tag, attrs.clone(), Some(parent));
                            
                            // Check if it's a void element that shouldn't be pushed to stack
                            let is_void = VOID_ELEMENTS.contains(&tag.as_str());
                            
                            if !*self_closing && !is_void {
                                stack.push(id);
                            }
                        }
                    }
                }

                Token::EndTag { name } => {
                    // Flush pending text before processing end tag
                    if let Some(&parent) = stack.last() {
                        self.flush_pending_text(&mut dom, parent);
                    }
                    
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
            }
        }

        dom
    }
}

#[cfg(test)]
mod debug_tests {
    use super::*;

    fn print_dom_tree(dom: &Dom, node_id: NodeId, indent: usize) {
        let node = &dom.nodes[node_id];
        let prefix = "  ".repeat(indent);
        
        match &node.node_type {
            crate::dom::NodeType::Element(el) => {
                let attrs: Vec<String> = el.attributes.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                if attrs.is_empty() {
                    eprintln!("{}<{}>", prefix, el.tag_name);
                } else {
                    eprintln!("{}<{} {}>", prefix, el.tag_name, attrs.join(" "));
                }
            }
            crate::dom::NodeType::Text(text) => {
                let text = text.trim();
                if !text.is_empty() {
                    let display = if text.len() > 40 {
                        format!("{}...", &text[..40])
                    } else {
                        text.to_string()
                    };
                    eprintln!("{}TEXT: {:?}", prefix, display);
                }
            }
        }
        
        for &child_id in &node.children {
            print_dom_tree(dom, child_id, indent + 1);
        }
    }

    #[test]
    fn test_parse_old_cern_html() {
        let html = r#"<TITLE>What is Hypertext?</TITLE>
<NEXTID 20>
<H1>What is HyperText</H1>Hypertext is text which is not constrained to be linear.<P>
Hypertext is text which contains <A NAME=0 HREF=Terms.html#link>links</A> to other texts.<P>
See also:
<UL>
<LI><A NAME=2 HREF=Terms.html>A list of terms</A> used in hypertext litterature.
<LI><A NAME=19 HREF=../Conferences/Overview.html>Conferences</A>
</UL>"#;

        eprintln!("\n=== PARSING OLD CERN HTML ===");
        let parser = HtmlParser::new(html);
        let dom = parser.parse();
        
        eprintln!("\n=== ACTUAL DOM STRUCTURE ===");
        print_dom_tree(&dom, 0, 0);
        
        eprintln!("\n=== NODE COUNT: {} ===", dom.nodes.len());
    }
}
