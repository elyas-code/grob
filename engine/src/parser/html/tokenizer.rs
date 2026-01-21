#[derive(Debug, Clone)]
pub enum Token {
    Doctype {
        name: String,
    },
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
        self_closing: bool,
    },
    EndTag {
        name: String,
    },
    Comment(String),
    Text(String),
    Eof,
}

pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

// Void elements that don't have closing tags
const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input",
    "link", "meta", "param", "source", "track", "wbr",
];

impl Tokenizer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    fn peek_ahead(&self, n: usize) -> Option<char> {
        self.input.get(self.pos + n).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.peek();
        self.pos += 1;
        c
    }

    fn consume_while<F>(&mut self, test: F) -> String
    where
        F: Fn(char) -> bool,
    {
        let mut result = String::new();
        while let Some(c) = self.peek() {
            if !test(c) {
                break;
            }
            result.push(c);
            self.next();
        }
        result
    }

    fn consume_whitespace(&mut self) {
        self.consume_while(|c| c.is_whitespace());
    }

    fn parse_attribute(&mut self) -> Option<(String, String)> {
        self.consume_whitespace();

        // Consume attribute name (allow hyphens and underscores)
        let key = self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ':');
        if key.is_empty() {
            return None;
        }

        self.consume_whitespace();

        if self.peek() == Some('=') {
            self.next();
            self.consume_whitespace();

            // Handle quoted attributes
            let value = match self.peek() {
                Some('"') => {
                    self.next();
                    let val = self.consume_while(|c| c != '"');
                    self.next(); // consume closing quote
                    val
                }
                Some('\'') => {
                    self.next();
                    let val = self.consume_while(|c| c != '\'');
                    self.next(); // consume closing quote
                    val
                }
                Some(_) => {
                    // Unquoted attribute value
                    self.consume_while(|c| !c.is_whitespace() && c != '>' && c != '"' && c != '\'')
                }
                None => String::new(),
            };

            Some((key, value))
        } else {
            // Boolean attribute
            Some((key.clone(), key))
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.consume_whitespace();

        if self.peek().is_none() {
            return Some(Token::Eof);
        }

        if self.peek() == Some('<') {
            self.next();

            // Comment: <!--
            if self.peek() == Some('!')
                && self.peek_ahead(1) == Some('-')
                && self.peek_ahead(2) == Some('-')
            {
                self.next(); // !
                self.next(); // -
                self.next(); // -

                let mut comment = String::new();
                while self.peek().is_some() {
                    if self.peek() == Some('-') && self.peek_ahead(1) == Some('-') && self.peek_ahead(2) == Some('>') {
                        break;
                    }
                    if let Some(c) = self.next() {
                        comment.push(c);
                    }
                }

                // Consume closing -->
                self.next(); // -
                self.next(); // -
                self.next(); // >

                return Some(Token::Comment(comment));
            }

            // Doctype: <!
            if self.peek() == Some('!') {
                self.next();
                let doctype_str = self.consume_while(|c| c != '>');
                self.next(); // consume >

                if doctype_str.to_lowercase().starts_with("doctype") {
                    let name = doctype_str
                        .split_whitespace()
                        .nth(1)
                        .unwrap_or("html")
                        .to_lowercase();
                    return Some(Token::Doctype { name });
                }
            }

            // End tag: </
            if self.peek() == Some('/') {
                self.next();
                let name = self.consume_while(|c| c.is_alphanumeric());
                self.consume_whitespace();
                self.next(); // consume >
                return Some(Token::EndTag { name });
            }

            // Start tag
            let name = self.consume_while(|c| c.is_alphanumeric());
            let mut attributes = Vec::new();

            // Parse attributes
            loop {
                self.consume_whitespace();
                match self.peek() {
                    Some('>') => {
                        self.next();
                        let self_closing = VOID_ELEMENTS.contains(&name.to_lowercase().as_str());
                        return Some(Token::StartTag {
                            name,
                            attributes,
                            self_closing,
                        });
                    }
                    Some('/') => {
                        self.next();
                        if self.peek() == Some('>') {
                            self.next();
                            return Some(Token::StartTag {
                                name,
                                attributes,
                                self_closing: true,
                            });
                        }
                    }
                    Some(_) => {
                        if let Some(attr) = self.parse_attribute() {
                            attributes.push(attr);
                        }
                    }
                    None => {
                        return Some(Token::StartTag {
                            name,
                            attributes,
                            self_closing: false,
                        })
                    }
                }
            }
        } else {
            // Text content
            let text = self.consume_while(|c| c != '<');
            if !text.is_empty() {
                Some(Token::Text(text))
            } else {
                self.next_token()
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            if matches!(token, Token::Eof) {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}
