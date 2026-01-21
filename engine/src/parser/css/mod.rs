#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    // Selectors
    Ident(String),
    Hash(String),      // #id
    Dot(String),       // .class
    Asterisk,          // *
    Plus,              // +
    Greater,           // >
    Tilde,             // ~
    Pipe,              // |

    // Delimiters
    OpenBrace,         // {
    CloseBrace,        // }
    OpenParen,         // (
    CloseParen,        // )
    OpenBracket,       // [
    CloseBracket,      // ]
    Colon,             // :
    DoubleColon,       // ::
    Semicolon,         // ;
    Comma,             // ,

    // Values
    String(String),
    Url(String),
    Number(f32),
    Dimension {
        value: f32,
        unit: String,  // px, em, %, etc.
    },
    Percentage(f32),
    Color(String),     // #fff, #ffffff, rgb(), etc.

    // Operators
    Equals,            // =
    Includes,          // ~=
    DashMatch,         // |=
    SubstringMatch,    // *=
    PrefixMatch,       // ^=
    SuffixMatch,       // $=

    // Keywords
    At(String),        // @media, @import, etc.
    Function(String),  // rgb(, calc(, etc.

    // Special
    Comment(String),
    Whitespace,
    Eof,
}

pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl CssTokenizer {
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

    pub fn next_token(&mut self) -> Option<CssToken> {
        match self.peek() {
            None => return Some(CssToken::Eof),

            // Whitespace
            Some(c) if c.is_whitespace() => {
                self.consume_while(|c| c.is_whitespace());
                return self.next_token();
            }

            // Comments
            Some('/') if self.peek_ahead(1) == Some('*') => {
                self.next(); // /
                self.next(); // *
                let mut comment = String::new();
                while !(self.peek() == Some('*') && self.peek_ahead(1) == Some('/')) {
                    if let Some(c) = self.next() {
                        comment.push(c);
                    } else {
                        break;
                    }
                }
                self.next(); // *
                self.next(); // /
                return self.next_token(); // Skip comments
            }

            // Strings
            Some('"') => {
                self.next();
                let string = self.consume_while(|c| c != '"');
                self.next();
                return Some(CssToken::String(string));
            }

            Some('\'') => {
                self.next();
                let string = self.consume_while(|c| c != '\'');
                self.next();
                return Some(CssToken::String(string));
            }

            // URLs
            Some('u') | Some('U') if matches!(self.peek_ahead(1), Some('r') | Some('R')) && matches!(self.peek_ahead(2), Some('l') | Some('L')) => {
                self.next(); // u
                self.next(); // r
                self.next(); // l
                self.next(); // (
                self.consume_while(|c| c.is_whitespace());
                
                let url = if self.peek() == Some('"') || self.peek() == Some('\'') {
                    let quote = self.next().unwrap();
                    let url_str = self.consume_while(|c| c != quote);
                    self.next();
                    url_str
                } else {
                    self.consume_while(|c| c != ')' && !c.is_whitespace())
                };
                
                self.consume_while(|c| c.is_whitespace());
                self.next(); // )
                return Some(CssToken::Url(url));
            }

            // Numbers and dimensions
            Some(c) if c.is_ascii_digit() || (c == '.' && matches!(self.peek_ahead(1), Some(d) if d.is_ascii_digit())) => {
                let num_str = self.consume_while(|c| c.is_ascii_digit() || c == '.');
                let num: f32 = num_str.parse().unwrap_or(0.0);

                // Check for percentage
                if self.peek() == Some('%') {
                    self.next();
                    return Some(CssToken::Percentage(num));
                }

                // Check for dimension (unit)
                if let Some(c) = self.peek() {
                    if c.is_alphabetic() {
                        let unit = self.consume_while(|c| c.is_alphabetic());
                        return Some(CssToken::Dimension {
                            value: num,
                            unit: unit.to_lowercase(),
                        });
                    }
                }

                return Some(CssToken::Number(num));
            }

            // Hash/Color
            Some('#') => {
                self.next();
                let hex_or_id = self.consume_while(|c| c.is_alphanumeric());
                
                if hex_or_id.len() <= 6 && hex_or_id.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(CssToken::Color(format!("#{}", hex_or_id)));
                } else {
                    return Some(CssToken::Hash(hex_or_id));
                }
            }

            // Class
            Some('.') if matches!(self.peek_ahead(1), Some(c) if c.is_alphabetic()) => {
                self.next();
                let class = self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_');
                return Some(CssToken::Dot(class));
            }

            // At-rules
            Some('@') => {
                self.next();
                let at_rule = self.consume_while(|c| c.is_alphabetic() || c == '-');
                return Some(CssToken::At(at_rule));
            }

            // Operators and delimiters
            Some('{') => { self.next(); return Some(CssToken::OpenBrace); }
            Some('}') => { self.next(); return Some(CssToken::CloseBrace); }
            Some('(') => { self.next(); return Some(CssToken::OpenParen); }
            Some(')') => { self.next(); return Some(CssToken::CloseParen); }
            Some('[') => { self.next(); return Some(CssToken::OpenBracket); }
            Some(']') => { self.next(); return Some(CssToken::CloseBracket); }
            Some(';') => { self.next(); return Some(CssToken::Semicolon); }
            Some(',') => { self.next(); return Some(CssToken::Comma); }
            Some('>') => { self.next(); return Some(CssToken::Greater); }
            Some('+') => { self.next(); return Some(CssToken::Plus); }
            Some('~') if self.peek_ahead(1) == Some('=') => {
                self.next(); self.next();
                return Some(CssToken::Includes);
            }
            Some('~') => { self.next(); return Some(CssToken::Tilde); }
            Some('|') if self.peek_ahead(1) == Some('=') => {
                self.next(); self.next();
                return Some(CssToken::DashMatch);
            }
            Some('|') => { self.next(); return Some(CssToken::Pipe); }
            Some(':') if self.peek_ahead(1) == Some(':') => {
                self.next(); self.next();
                return Some(CssToken::DoubleColon);
            }
            Some(':') => { self.next(); return Some(CssToken::Colon); }
            Some('*') if self.peek_ahead(1) == Some('=') => {
                self.next(); self.next();
                return Some(CssToken::SubstringMatch);
            }
            Some('*') => { self.next(); return Some(CssToken::Asterisk); }
            Some('=') => { self.next(); return Some(CssToken::Equals); }
            Some('^') if self.peek_ahead(1) == Some('=') => {
                self.next(); self.next();
                return Some(CssToken::PrefixMatch);
            }
            Some('$') if self.peek_ahead(1) == Some('=') => {
                self.next(); self.next();
                return Some(CssToken::SuffixMatch);
            }

            // Identifiers
            Some(c) if c.is_alphabetic() || c == '-' || c == '_' => {
                let ident = self.consume_while(|c| c.is_alphanumeric() || c == '-' || c == '_');

                // Check if it's followed by ( to make it a function
                if self.peek() == Some('(') {
                    return Some(CssToken::Function(ident));
                }

                return Some(CssToken::Ident(ident));
            }

            _ => {
                self.next();
                self.next_token()
            }
        }
    }

    pub fn tokenize(&mut self) -> Vec<CssToken> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token() {
            if matches!(token, CssToken::Eof) {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

pub mod parser;
pub use parser::{CssParser, Selector, Rule, Declaration, CssItem};
