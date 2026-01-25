// HTML Tokenizer implementation following WHATWG HTML Living Standard
// Spec Reference: https://html.spec.whatwg.org/multipage/parsing.html#tokenization
//
// IMPLEMENTATION STATUS:
// ✅ Data state - basic
// ✅ Tag open state
// ✅ End tag open state
// ✅ Tag name state
// ✅ Before attribute name state
// ✅ Attribute name state
// ✅ After attribute name state
// ✅ Before attribute value state
// ✅ Attribute value (double-quoted) state
// ✅ Attribute value (single-quoted) state
// ✅ Attribute value (unquoted) state
// ✅ After attribute value (quoted) state
// ⚠️ Bogus comment state - partial
// ✅ Markup declaration open state - basic
// ⚠️ Comment start state - basic
// ⚠️ Comment state - basic
// ⚠️ Comment end state - basic
// ⚠️ DOCTYPE state - basic
// ❌ Character reference states - not implemented
// ❌ RCDATA states - not implemented
// ❌ RAWTEXT states - not implemented
// ❌ Script data states - not implemented
//
// TODO(spec 13.2.5.1): Implement preprocessing input stream
// TODO(spec 13.2.5.2): Implement parse errors properly

use std::collections::VecDeque;

/// Debug logging for tokenizer operations
const DEBUG_TOKENIZER: bool = false;

fn tokenizer_log(msg: &str) {
    if DEBUG_TOKENIZER {
        eprintln!("[TOKENIZER] {}", msg);
    }
}

/// Token types per spec 13.2.5
/// Reference: https://html.spec.whatwg.org/multipage/parsing.html#tokenization
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// DOCTYPE token (spec 13.2.5)
    Doctype {
        name: Option<String>,
        public_id: Option<String>,
        system_id: Option<String>,
        force_quirks: bool,
    },
    /// Start tag token (spec 13.2.5)
    StartTag {
        name: String,
        attributes: Vec<Attribute>,
        self_closing: bool,
    },
    /// End tag token (spec 13.2.5)
    EndTag {
        name: String,
    },
    /// Comment token (spec 13.2.5)
    Comment(String),
    /// Character token (spec 13.2.5)
    Character(char),
    /// End-of-file token (spec 13.2.5)
    Eof,
}

/// Attribute structure
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

/// Tokenizer states per spec 13.2.5.1
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenizerState {
    Data,
    RcData,
    RawText,
    ScriptData,
    PlainText,
    TagOpen,
    EndTagOpen,
    TagName,
    RcDataLessThan,
    RcDataEndTagOpen,
    RcDataEndTagName,
    RawTextLessThan,
    RawTextEndTagOpen,
    RawTextEndTagName,
    ScriptDataLessThan,
    BeforeAttributeName,
    AttributeName,
    AfterAttributeName,
    BeforeAttributeValue,
    AttributeValueDoubleQuoted,
    AttributeValueSingleQuoted,
    AttributeValueUnquoted,
    AfterAttributeValueQuoted,
    SelfClosingStartTag,
    BogusComment,
    MarkupDeclarationOpen,
    CommentStart,
    CommentStartDash,
    Comment,
    CommentLessThan,
    CommentEndDash,
    CommentEnd,
    CommentEndBang,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    AfterDoctypeName,
}

/// Void elements that cannot have content (spec 13.1.2)
/// Also includes obsolete HTML elements that are void/empty
pub const VOID_ELEMENTS: &[&str] = &[
    // Standard HTML5 void elements
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
    // Obsolete void elements (from older HTML versions)
    "basefont", "bgsound", "frame", "isindex", "keygen", "nextid",
    "command", "spacer",
];

/// Raw text elements (spec 13.1.2.1)
pub const RAW_TEXT_ELEMENTS: &[&str] = &["script", "style"];

/// Escapable raw text elements (spec 13.1.2.2)
pub const ESCAPABLE_RAW_TEXT_ELEMENTS: &[&str] = &["textarea", "title"];

/// HTML Tokenizer
pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
    state: TokenizerState,
    current_token: Option<Token>,
    token_queue: VecDeque<Token>,
    temp_buffer: String,
    current_attribute: Option<Attribute>,
    last_start_tag_name: Option<String>,
    reconsume: bool,
}

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
            state: TokenizerState::Data,
            current_token: None,
            token_queue: VecDeque::new(),
            temp_buffer: String::new(),
            current_attribute: None,
            last_start_tag_name: None,
            reconsume: false,
        }
    }

    pub fn state(&self) -> TokenizerState {
        self.state
    }

    pub fn set_state(&mut self, state: TokenizerState) {
        tokenizer_log(&format!("State transition: {:?} -> {:?}", self.state, state));
        self.state = state;
    }

    fn consume_next(&mut self) -> Option<char> {
        if self.reconsume {
            self.reconsume = false;
            return self.current_input_char();
        }
        let c = self.input.get(self.pos).copied();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn current_input_char(&self) -> Option<char> {
        if self.pos > 0 {
            self.input.get(self.pos - 1).copied()
        } else {
            None
        }
    }

    fn next_chars_are_case_insensitive(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            match self.input.get(self.pos + i) {
                Some(&input_char) => {
                    if input_char.to_ascii_lowercase() != c.to_ascii_lowercase() {
                        return false;
                    }
                }
                None => return false,
            }
        }
        true
    }

    fn consume_chars(&mut self, count: usize) {
        for _ in 0..count {
            self.consume_next();
        }
    }

    fn reconsume_in(&mut self, state: TokenizerState) {
        self.reconsume = true;
        self.state = state;
    }

    fn emit_char(&mut self, c: char) {
        tokenizer_log(&format!("Emit character: {:?}", c));
        self.token_queue.push_back(Token::Character(c));
    }

    fn emit_current_token(&mut self) {
        if let Some(token) = self.current_token.take() {
            tokenizer_log(&format!("Emit token: {:?}", token));
            if let Token::StartTag { ref name, .. } = token {
                self.last_start_tag_name = Some(name.clone());
            }
            self.token_queue.push_back(token);
        }
    }

    fn emit_eof(&mut self) {
        tokenizer_log("Emit EOF");
        self.token_queue.push_back(Token::Eof);
    }

    fn create_start_tag(&mut self) {
        self.current_token = Some(Token::StartTag {
            name: String::new(),
            attributes: Vec::new(),
            self_closing: false,
        });
    }

    fn create_end_tag(&mut self) {
        self.current_token = Some(Token::EndTag {
            name: String::new(),
        });
    }

    fn create_comment(&mut self, data: &str) {
        self.current_token = Some(Token::Comment(data.to_string()));
    }

    fn create_doctype(&mut self) {
        self.current_token = Some(Token::Doctype {
            name: None,
            public_id: None,
            system_id: None,
            force_quirks: false,
        });
    }

    fn append_to_tag_name(&mut self, c: char) {
        if let Some(token) = &mut self.current_token {
            match token {
                Token::StartTag { name, .. } => {
                    name.push(c.to_ascii_lowercase());
                }
                Token::EndTag { name, .. } => {
                    name.push(c.to_ascii_lowercase());
                }
                _ => {}
            }
        }
    }

    fn start_new_attribute(&mut self) {
        self.finalize_current_attribute();
        self.current_attribute = Some(Attribute {
            name: String::new(),
            value: String::new(),
        });
    }

    fn append_to_attribute_name(&mut self, c: char) {
        if let Some(attr) = &mut self.current_attribute {
            attr.name.push(c.to_ascii_lowercase());
        }
    }

    fn append_to_attribute_value(&mut self, c: char) {
        if let Some(attr) = &mut self.current_attribute {
            attr.value.push(c);
        }
    }

    fn finalize_current_attribute(&mut self) {
        if let Some(attr) = self.current_attribute.take() {
            if let Some(Token::StartTag { attributes, .. }) = &mut self.current_token {
                if !attributes.iter().any(|a| a.name == attr.name) {
                    attributes.push(attr);
                } else {
                    tokenizer_log(&format!("Parse error: duplicate attribute '{}'", attr.name));
                }
            }
        }
    }

    fn append_to_comment(&mut self, c: char) {
        if let Some(Token::Comment(ref mut data)) = self.current_token {
            data.push(c);
        }
    }

    fn set_self_closing(&mut self) {
        if let Some(Token::StartTag { self_closing, .. }) = &mut self.current_token {
            *self_closing = true;
        }
    }

    fn append_to_doctype_name(&mut self, c: char) {
        if let Some(Token::Doctype { name, .. }) = &mut self.current_token {
            if name.is_none() {
                *name = Some(String::new());
            }
            if let Some(ref mut n) = name {
                n.push(c.to_ascii_lowercase());
            }
        }
    }

    fn set_force_quirks(&mut self) {
        if let Some(Token::Doctype { force_quirks, .. }) = &mut self.current_token {
            *force_quirks = true;
        }
    }

    fn is_appropriate_end_tag(&self) -> bool {
        if let Some(Token::EndTag { name, .. }) = &self.current_token {
            if let Some(ref last) = self.last_start_tag_name {
                return name == last;
            }
        }
        false
    }

    pub fn next_token(&mut self) -> Option<Token> {
        loop {
            if let Some(token) = self.token_queue.pop_front() {
                return Some(token);
            }

            let c = self.consume_next();
            
            match self.state {
                TokenizerState::Data => self.data_state(c),
                TokenizerState::RcData => self.rcdata_state(c),
                TokenizerState::RawText => self.rawtext_state(c),
                TokenizerState::TagOpen => self.tag_open_state(c),
                TokenizerState::EndTagOpen => self.end_tag_open_state(c),
                TokenizerState::TagName => self.tag_name_state(c),
                TokenizerState::RcDataLessThan => self.rcdata_less_than_state(c),
                TokenizerState::RcDataEndTagOpen => self.rcdata_end_tag_open_state(c),
                TokenizerState::RcDataEndTagName => self.rcdata_end_tag_name_state(c),
                TokenizerState::RawTextLessThan => self.rawtext_less_than_state(c),
                TokenizerState::RawTextEndTagOpen => self.rawtext_end_tag_open_state(c),
                TokenizerState::RawTextEndTagName => self.rawtext_end_tag_name_state(c),
                TokenizerState::BeforeAttributeName => self.before_attribute_name_state(c),
                TokenizerState::AttributeName => self.attribute_name_state(c),
                TokenizerState::AfterAttributeName => self.after_attribute_name_state(c),
                TokenizerState::BeforeAttributeValue => self.before_attribute_value_state(c),
                TokenizerState::AttributeValueDoubleQuoted => self.attribute_value_double_quoted_state(c),
                TokenizerState::AttributeValueSingleQuoted => self.attribute_value_single_quoted_state(c),
                TokenizerState::AttributeValueUnquoted => self.attribute_value_unquoted_state(c),
                TokenizerState::AfterAttributeValueQuoted => self.after_attribute_value_quoted_state(c),
                TokenizerState::SelfClosingStartTag => self.self_closing_start_tag_state(c),
                TokenizerState::BogusComment => self.bogus_comment_state(c),
                TokenizerState::MarkupDeclarationOpen => self.markup_declaration_open_state(c),
                TokenizerState::CommentStart => self.comment_start_state(c),
                TokenizerState::CommentStartDash => self.comment_start_dash_state(c),
                TokenizerState::Comment => self.comment_state(c),
                TokenizerState::CommentEndDash => self.comment_end_dash_state(c),
                TokenizerState::CommentEnd => self.comment_end_state(c),
                TokenizerState::CommentEndBang => self.comment_end_bang_state(c),
                TokenizerState::Doctype => self.doctype_state(c),
                TokenizerState::BeforeDoctypeName => self.before_doctype_name_state(c),
                TokenizerState::DoctypeName => self.doctype_name_state(c),
                TokenizerState::AfterDoctypeName => self.after_doctype_name_state(c),
                _ => {
                    tokenizer_log(&format!("Unimplemented state: {:?}", self.state));
                    self.state = TokenizerState::Data;
                }
            }
        }
    }

    /// 13.2.5.1 Data state
    fn data_state(&mut self, c: Option<char>) {
        match c {
            Some('&') => {
                // TODO: character reference
                self.emit_char('&');
            }
            Some('<') => {
                self.state = TokenizerState::TagOpen;
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.emit_char('\0');
            }
            None => {
                self.emit_eof();
            }
            Some(c) => {
                self.emit_char(c);
            }
        }
    }

    /// 13.2.5.2 RCDATA state
    fn rcdata_state(&mut self, c: Option<char>) {
        match c {
            Some('&') => {
                self.emit_char('&');
            }
            Some('<') => {
                self.state = TokenizerState::RcDataLessThan;
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_eof();
            }
            Some(c) => {
                self.emit_char(c);
            }
        }
    }

    /// 13.2.5.3 RAWTEXT state
    fn rawtext_state(&mut self, c: Option<char>) {
        match c {
            Some('<') => {
                self.state = TokenizerState::RawTextLessThan;
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.emit_char('\u{FFFD}');
            }
            None => {
                self.emit_eof();
            }
            Some(c) => {
                self.emit_char(c);
            }
        }
    }

    /// 13.2.5.6 Tag open state
    fn tag_open_state(&mut self, c: Option<char>) {
        match c {
            Some('!') => {
                self.state = TokenizerState::MarkupDeclarationOpen;
            }
            Some('/') => {
                self.state = TokenizerState::EndTagOpen;
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_start_tag();
                self.reconsume_in(TokenizerState::TagName);
            }
            Some('?') => {
                tokenizer_log("Parse error: unexpected-question-mark-instead-of-tag-name");
                self.create_comment("");
                self.reconsume_in(TokenizerState::BogusComment);
            }
            None => {
                tokenizer_log("Parse error: eof-before-tag-name");
                self.emit_char('<');
                self.emit_eof();
            }
            Some(_) => {
                tokenizer_log("Parse error: invalid-first-character-of-tag-name");
                self.emit_char('<');
                self.reconsume_in(TokenizerState::Data);
            }
        }
    }

    /// 13.2.5.7 End tag open state
    fn end_tag_open_state(&mut self, c: Option<char>) {
        match c {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.reconsume_in(TokenizerState::TagName);
            }
            Some('>') => {
                tokenizer_log("Parse error: missing-end-tag-name");
                self.state = TokenizerState::Data;
            }
            None => {
                tokenizer_log("Parse error: eof-before-tag-name");
                self.emit_char('<');
                self.emit_char('/');
                self.emit_eof();
            }
            Some(_) => {
                tokenizer_log("Parse error: invalid-first-character-of-tag-name");
                self.create_comment("");
                self.reconsume_in(TokenizerState::BogusComment);
            }
        }
    }

    /// 13.2.5.8 Tag name state
    fn tag_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = TokenizerState::BeforeAttributeName;
            }
            Some('/') => {
                self.state = TokenizerState::SelfClosingStartTag;
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_tag_name('\u{FFFD}');
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(c) => {
                self.append_to_tag_name(c);
            }
        }
    }

    /// 13.2.5.9 RCDATA less-than sign state
    fn rcdata_less_than_state(&mut self, c: Option<char>) {
        match c {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = TokenizerState::RcDataEndTagOpen;
            }
            _ => {
                self.emit_char('<');
                self.reconsume_in(TokenizerState::RcData);
            }
        }
    }

    /// 13.2.5.10 RCDATA end tag open state
    fn rcdata_end_tag_open_state(&mut self, c: Option<char>) {
        match c {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.reconsume_in(TokenizerState::RcDataEndTagName);
            }
            _ => {
                self.emit_char('<');
                self.emit_char('/');
                self.reconsume_in(TokenizerState::RcData);
            }
        }
    }

    /// 13.2.5.11 RCDATA end tag name state
    fn rcdata_end_tag_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = TokenizerState::BeforeAttributeName;
                    return;
                }
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = TokenizerState::SelfClosingStartTag;
                    return;
                }
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = TokenizerState::Data;
                    self.emit_current_token();
                    return;
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.append_to_tag_name(c);
                self.temp_buffer.push(c);
                return;
            }
            _ => {}
        }
        self.emit_char('<');
        self.emit_char('/');
        let temp_chars: Vec<char> = self.temp_buffer.chars().collect();
        for c in temp_chars {
            self.emit_char(c);
        }
        self.reconsume_in(TokenizerState::RcData);
    }

    /// 13.2.5.12 RAWTEXT less-than sign state
    fn rawtext_less_than_state(&mut self, c: Option<char>) {
        match c {
            Some('/') => {
                self.temp_buffer.clear();
                self.state = TokenizerState::RawTextEndTagOpen;
            }
            _ => {
                self.emit_char('<');
                self.reconsume_in(TokenizerState::RawText);
            }
        }
    }

    /// 13.2.5.13 RAWTEXT end tag open state
    fn rawtext_end_tag_open_state(&mut self, c: Option<char>) {
        match c {
            Some(c) if c.is_ascii_alphabetic() => {
                self.create_end_tag();
                self.reconsume_in(TokenizerState::RawTextEndTagName);
            }
            _ => {
                self.emit_char('<');
                self.emit_char('/');
                self.reconsume_in(TokenizerState::RawText);
            }
        }
    }

    /// 13.2.5.14 RAWTEXT end tag name state
    fn rawtext_end_tag_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                if self.is_appropriate_end_tag() {
                    self.state = TokenizerState::BeforeAttributeName;
                    return;
                }
            }
            Some('/') => {
                if self.is_appropriate_end_tag() {
                    self.state = TokenizerState::SelfClosingStartTag;
                    return;
                }
            }
            Some('>') => {
                if self.is_appropriate_end_tag() {
                    self.state = TokenizerState::Data;
                    self.emit_current_token();
                    return;
                }
            }
            Some(c) if c.is_ascii_alphabetic() => {
                self.append_to_tag_name(c);
                self.temp_buffer.push(c);
                return;
            }
            _ => {}
        }
        self.emit_char('<');
        self.emit_char('/');
        let temp_chars: Vec<char> = self.temp_buffer.chars().collect();
        for c in temp_chars {
            self.emit_char(c);
        }
        self.reconsume_in(TokenizerState::RawText);
    }

    /// 13.2.5.32 Before attribute name state
    fn before_attribute_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('/') | Some('>') | None => {
                self.reconsume_in(TokenizerState::AfterAttributeName);
            }
            Some('=') => {
                tokenizer_log("Parse error: unexpected-equals-sign-before-attribute-name");
                self.start_new_attribute();
                self.append_to_attribute_name('=');
                self.state = TokenizerState::AttributeName;
            }
            Some(_) => {
                self.start_new_attribute();
                self.reconsume_in(TokenizerState::AttributeName);
            }
        }
    }

    /// 13.2.5.33 Attribute name state
    fn attribute_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') | Some('/') | Some('>') | None => {
                self.reconsume_in(TokenizerState::AfterAttributeName);
            }
            Some('=') => {
                self.state = TokenizerState::BeforeAttributeValue;
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_attribute_name('\u{FFFD}');
            }
            Some('"') | Some('\'') | Some('<') => {
                tokenizer_log("Parse error: unexpected-character-in-attribute-name");
                self.append_to_attribute_name(c.unwrap());
            }
            Some(c) => {
                self.append_to_attribute_name(c);
            }
        }
    }

    /// 13.2.5.34 After attribute name state
    fn after_attribute_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('/') => {
                self.finalize_current_attribute();
                self.state = TokenizerState::SelfClosingStartTag;
            }
            Some('=') => {
                self.state = TokenizerState::BeforeAttributeValue;
            }
            Some('>') => {
                self.finalize_current_attribute();
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(_) => {
                self.finalize_current_attribute();
                self.start_new_attribute();
                self.reconsume_in(TokenizerState::AttributeName);
            }
        }
    }

    /// 13.2.5.35 Before attribute value state
    fn before_attribute_value_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('"') => {
                self.state = TokenizerState::AttributeValueDoubleQuoted;
            }
            Some('\'') => {
                self.state = TokenizerState::AttributeValueSingleQuoted;
            }
            Some('>') => {
                tokenizer_log("Parse error: missing-attribute-value");
                self.finalize_current_attribute();
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            Some(_) | None => {
                self.reconsume_in(TokenizerState::AttributeValueUnquoted);
            }
        }
    }

    /// 13.2.5.36 Attribute value (double-quoted) state
    fn attribute_value_double_quoted_state(&mut self, c: Option<char>) {
        match c {
            Some('"') => {
                self.state = TokenizerState::AfterAttributeValueQuoted;
            }
            Some('&') => {
                self.append_to_attribute_value('&');
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_attribute_value('\u{FFFD}');
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(c) => {
                self.append_to_attribute_value(c);
            }
        }
    }

    /// 13.2.5.37 Attribute value (single-quoted) state
    fn attribute_value_single_quoted_state(&mut self, c: Option<char>) {
        match c {
            Some('\'') => {
                self.state = TokenizerState::AfterAttributeValueQuoted;
            }
            Some('&') => {
                self.append_to_attribute_value('&');
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_attribute_value('\u{FFFD}');
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(c) => {
                self.append_to_attribute_value(c);
            }
        }
    }

    /// 13.2.5.38 Attribute value (unquoted) state
    fn attribute_value_unquoted_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.finalize_current_attribute();
                self.state = TokenizerState::BeforeAttributeName;
            }
            Some('&') => {
                self.append_to_attribute_value('&');
            }
            Some('>') => {
                self.finalize_current_attribute();
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_attribute_value('\u{FFFD}');
            }
            Some('"') | Some('\'') | Some('<') | Some('=') | Some('`') => {
                tokenizer_log("Parse error: unexpected-character-in-unquoted-attribute-value");
                self.append_to_attribute_value(c.unwrap());
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(c) => {
                self.append_to_attribute_value(c);
            }
        }
    }

    /// 13.2.5.39 After attribute value (quoted) state
    fn after_attribute_value_quoted_state(&mut self, c: Option<char>) {
        self.finalize_current_attribute();
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = TokenizerState::BeforeAttributeName;
            }
            Some('/') => {
                self.state = TokenizerState::SelfClosingStartTag;
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(_) => {
                tokenizer_log("Parse error: missing-whitespace-between-attributes");
                self.reconsume_in(TokenizerState::BeforeAttributeName);
            }
        }
    }

    /// 13.2.5.40 Self-closing start tag state
    fn self_closing_start_tag_state(&mut self, c: Option<char>) {
        match c {
            Some('>') => {
                self.set_self_closing();
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-tag");
                self.emit_eof();
            }
            Some(_) => {
                tokenizer_log("Parse error: unexpected-solidus-in-tag");
                self.reconsume_in(TokenizerState::BeforeAttributeName);
            }
        }
    }

    /// 13.2.5.41 Bogus comment state
    fn bogus_comment_state(&mut self, c: Option<char>) {
        match c {
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                self.emit_current_token();
                self.emit_eof();
            }
            Some('\0') => {
                self.append_to_comment('\u{FFFD}');
            }
            Some(c) => {
                self.append_to_comment(c);
            }
        }
    }

    /// 13.2.5.42 Markup declaration open state
    fn markup_declaration_open_state(&mut self, _c: Option<char>) {
        self.pos = self.pos.saturating_sub(1);
        
        if self.next_chars_are_case_insensitive("--") {
            self.consume_chars(2);
            self.create_comment("");
            self.state = TokenizerState::CommentStart;
        } else if self.next_chars_are_case_insensitive("DOCTYPE") {
            self.consume_chars(7);
            self.state = TokenizerState::Doctype;
        } else if self.next_chars_are_case_insensitive("[CDATA[") {
            tokenizer_log("Parse error: cdata-in-html-content");
            self.create_comment("[CDATA[");
            self.state = TokenizerState::BogusComment;
        } else {
            tokenizer_log("Parse error: incorrectly-opened-comment");
            self.create_comment("");
            self.state = TokenizerState::BogusComment;
        }
    }

    /// 13.2.5.43 Comment start state
    fn comment_start_state(&mut self, c: Option<char>) {
        match c {
            Some('-') => {
                self.state = TokenizerState::CommentStartDash;
            }
            Some('>') => {
                tokenizer_log("Parse error: abrupt-closing-of-empty-comment");
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            Some(_) | None => {
                self.reconsume_in(TokenizerState::Comment);
            }
        }
    }

    /// 13.2.5.44 Comment start dash state
    fn comment_start_dash_state(&mut self, c: Option<char>) {
        match c {
            Some('-') => {
                self.state = TokenizerState::CommentEnd;
            }
            Some('>') => {
                tokenizer_log("Parse error: abrupt-closing-of-empty-comment");
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-comment");
                self.emit_current_token();
                self.emit_eof();
            }
            Some(_) => {
                self.append_to_comment('-');
                self.reconsume_in(TokenizerState::Comment);
            }
        }
    }

    /// 13.2.5.45 Comment state
    fn comment_state(&mut self, c: Option<char>) {
        match c {
            Some('<') => {
                self.append_to_comment('<');
                self.state = TokenizerState::CommentLessThan;
            }
            Some('-') => {
                self.state = TokenizerState::CommentEndDash;
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_comment('\u{FFFD}');
            }
            None => {
                tokenizer_log("Parse error: eof-in-comment");
                self.emit_current_token();
                self.emit_eof();
            }
            Some(c) => {
                self.append_to_comment(c);
            }
        }
    }

    /// 13.2.5.50 Comment end dash state
    fn comment_end_dash_state(&mut self, c: Option<char>) {
        match c {
            Some('-') => {
                self.state = TokenizerState::CommentEnd;
            }
            None => {
                tokenizer_log("Parse error: eof-in-comment");
                self.emit_current_token();
                self.emit_eof();
            }
            Some(_) => {
                self.append_to_comment('-');
                self.reconsume_in(TokenizerState::Comment);
            }
        }
    }

    /// 13.2.5.51 Comment end state
    fn comment_end_state(&mut self, c: Option<char>) {
        match c {
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            Some('!') => {
                self.state = TokenizerState::CommentEndBang;
            }
            Some('-') => {
                self.append_to_comment('-');
            }
            None => {
                tokenizer_log("Parse error: eof-in-comment");
                self.emit_current_token();
                self.emit_eof();
            }
            Some(_) => {
                self.append_to_comment('-');
                self.append_to_comment('-');
                self.reconsume_in(TokenizerState::Comment);
            }
        }
    }

    /// 13.2.5.52 Comment end bang state
    fn comment_end_bang_state(&mut self, c: Option<char>) {
        match c {
            Some('-') => {
                self.append_to_comment('-');
                self.append_to_comment('-');
                self.append_to_comment('!');
                self.state = TokenizerState::CommentEndDash;
            }
            Some('>') => {
                tokenizer_log("Parse error: incorrectly-closed-comment");
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-comment");
                self.emit_current_token();
                self.emit_eof();
            }
            Some(_) => {
                self.append_to_comment('-');
                self.append_to_comment('-');
                self.append_to_comment('!');
                self.reconsume_in(TokenizerState::Comment);
            }
        }
    }

    /// 13.2.5.53 DOCTYPE state
    fn doctype_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = TokenizerState::BeforeDoctypeName;
            }
            Some('>') => {
                self.reconsume_in(TokenizerState::BeforeDoctypeName);
            }
            None => {
                tokenizer_log("Parse error: eof-in-doctype");
                self.create_doctype();
                self.set_force_quirks();
                self.emit_current_token();
                self.emit_eof();
            }
            Some(_) => {
                tokenizer_log("Parse error: missing-whitespace-before-doctype-name");
                self.reconsume_in(TokenizerState::BeforeDoctypeName);
            }
        }
    }

    /// 13.2.5.54 Before DOCTYPE name state
    fn before_doctype_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.create_doctype();
                self.append_to_doctype_name('\u{FFFD}');
                self.state = TokenizerState::DoctypeName;
            }
            Some('>') => {
                tokenizer_log("Parse error: missing-doctype-name");
                self.create_doctype();
                self.set_force_quirks();
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-doctype");
                self.create_doctype();
                self.set_force_quirks();
                self.emit_current_token();
                self.emit_eof();
            }
            Some(c) => {
                self.create_doctype();
                self.append_to_doctype_name(c);
                self.state = TokenizerState::DoctypeName;
            }
        }
    }

    /// 13.2.5.55 DOCTYPE name state
    fn doctype_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                self.state = TokenizerState::AfterDoctypeName;
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            Some('\0') => {
                tokenizer_log("Parse error: unexpected-null-character");
                self.append_to_doctype_name('\u{FFFD}');
            }
            None => {
                tokenizer_log("Parse error: eof-in-doctype");
                self.set_force_quirks();
                self.emit_current_token();
                self.emit_eof();
            }
            Some(c) => {
                self.append_to_doctype_name(c);
            }
        }
    }

    /// 13.2.5.56 After DOCTYPE name state
    fn after_doctype_name_state(&mut self, c: Option<char>) {
        match c {
            Some('\t') | Some('\n') | Some('\x0C') | Some(' ') => {
                // Ignore whitespace
            }
            Some('>') => {
                self.state = TokenizerState::Data;
                self.emit_current_token();
            }
            None => {
                tokenizer_log("Parse error: eof-in-doctype");
                self.set_force_quirks();
                self.emit_current_token();
                self.emit_eof();
            }
            Some(_) => {
                // TODO: Handle PUBLIC and SYSTEM identifiers
                self.set_force_quirks();
                self.state = TokenizerState::BogusComment;
            }
        }
    }

    /// Tokenize entire input (compatibility method)
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            match self.next_token() {
                Some(Token::Eof) => break,
                Some(token) => tokens.push(token),
                None => break,
            }
        }
        tokens
    }
}

impl Token {
    /// Get attributes as Vec<(String, String)> for compatibility
    pub fn get_attributes(&self) -> Vec<(String, String)> {
        match self {
            Token::StartTag { attributes, .. } => {
                attributes.iter().map(|a| (a.name.clone(), a.value.clone())).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Get tag name if this is a tag token
    pub fn tag_name(&self) -> Option<&str> {
        match self {
            Token::StartTag { name, .. } | Token::EndTag { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Check if this is a self-closing tag
    pub fn is_self_closing(&self) -> bool {
        matches!(self, Token::StartTag { self_closing: true, .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_element() {
        let mut tokenizer = Tokenizer::new("<div></div>");
        let tokens = tokenizer.tokenize();
        assert_eq!(tokens.len(), 2);
        assert!(matches!(&tokens[0], Token::StartTag { name, .. } if name == "div"));
        assert!(matches!(&tokens[1], Token::EndTag { name, .. } if name == "div"));
    }

    #[test]
    fn test_attributes() {
        let mut tokenizer = Tokenizer::new("<a href=\"https://example.com\" class='link'>text</a>");
        let tokens = tokenizer.tokenize();
        
        if let Token::StartTag { attributes, .. } = &tokens[0] {
            assert_eq!(attributes.len(), 2);
            assert_eq!(attributes[0].name, "href");
            assert_eq!(attributes[0].value, "https://example.com");
            assert_eq!(attributes[1].name, "class");
            assert_eq!(attributes[1].value, "link");
        } else {
            panic!("Expected start tag");
        }
    }

    #[test]
    fn test_self_closing() {
        let mut tokenizer = Tokenizer::new("<br/><img src=\"test.png\"/>");
        let tokens = tokenizer.tokenize();
        
        assert!(matches!(&tokens[0], Token::StartTag { self_closing: true, .. }));
        assert!(matches!(&tokens[1], Token::StartTag { self_closing: true, .. }));
    }

    #[test]
    fn test_doctype() {
        let mut tokenizer = Tokenizer::new("<!DOCTYPE html>");
        let tokens = tokenizer.tokenize();
        
        assert!(matches!(&tokens[0], Token::Doctype { name: Some(n), .. } if n == "html"));
    }

    #[test]
    fn test_comment() {
        let mut tokenizer = Tokenizer::new("<!-- comment -->");
        let tokens = tokenizer.tokenize();
        
        assert!(matches!(&tokens[0], Token::Comment(c) if c == " comment "));
    }

    #[test]
    fn test_anchor_tag() {
        let mut tokenizer = Tokenizer::new("<a href=\"https://example.com\">Click me</a>");
        let tokens = tokenizer.tokenize();
        
        // First token should be the start tag
        if let Token::StartTag { name, attributes, self_closing } = &tokens[0] {
            assert_eq!(name, "a");
            assert_eq!(attributes.len(), 1);
            assert_eq!(attributes[0].name, "href");
            assert_eq!(attributes[0].value, "https://example.com");
            assert!(!self_closing);
        } else {
            panic!("Expected StartTag, got {:?}", tokens[0]);
        }
        
        // Middle tokens should be characters
        let text: String = tokens[1..tokens.len()-1].iter()
            .filter_map(|t| if let Token::Character(c) = t { Some(*c) } else { None })
            .collect();
        assert_eq!(text, "Click me");
        
        // Last token should be end tag
        assert!(matches!(&tokens[tokens.len()-1], Token::EndTag { name } if name == "a"));
    }
}
