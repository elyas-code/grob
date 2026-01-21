use super::CssToken;

#[derive(Debug, Clone, PartialEq)]
pub enum Selector {
    Universal,                           // *
    Element(String),                     // div
    Id(String),                          // #myid
    Class(String),                       // .myclass
    Attribute {                          // [attr], [attr="value"]
        name: String,
        operator: Option<AttrOperator>,
        value: Option<String>,
    },
    PseudoClass(String),                // :hover, :focus, etc.
    PseudoElement(String),              // ::before, ::after, etc.
    Descendant(Box<Selector>, Box<Selector>),    // div p
    Child(Box<Selector>, Box<Selector>),        // div > p
    Adjacent(Box<Selector>, Box<Selector>),     // h1 + p
    GeneralSibling(Box<Selector>, Box<Selector>), // h1 ~ p
}

#[derive(Debug, Clone, PartialEq)]
pub enum AttrOperator {
    Exact,        // =
    Contains,     // ~=
    Dash,         // |=
    Substring,    // *=
    Prefix,       // ^=
    Suffix,       // $=
}

#[derive(Debug, Clone)]
pub struct Declaration {
    pub property: String,
    pub value: String,
    pub important: bool,
}

#[derive(Debug, Clone)]
pub struct Rule {
    pub selector: Selector,
    pub declarations: Vec<Declaration>,
}

#[derive(Debug, Clone)]
pub enum CssItem {
    Rule(Rule),
    AtRule {
        name: String,
        prelude: String,
        content: Vec<CssItem>,
    },
}

pub struct CssParser {
    tokens: Vec<CssToken>,
    pos: usize,
}

impl CssParser {
    pub fn new(tokens: Vec<CssToken>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&CssToken> {
        self.tokens.get(self.pos)
    }

    fn next(&mut self) -> Option<&CssToken> {
        let token = self.tokens.get(self.pos);
        self.pos += 1;
        token
    }

    fn expect(&mut self, expected: &CssToken) -> bool {
        if self.peek() == Some(expected) {
            self.next();
            true
        } else {
            false
        }
    }

    pub fn parse(&mut self) -> Vec<CssItem> {
        let mut items = Vec::new();

        while self.peek().is_some() {
            match self.peek() {
                Some(CssToken::At(_)) => {
                    items.push(self.parse_at_rule());
                }
                _ => {
                    if let Some(rule) = self.parse_rule() {
                        items.push(CssItem::Rule(rule));
                    } else {
                        self.next();
                    }
                }
            }
        }

        items
    }

    fn parse_at_rule(&mut self) -> CssItem {
        let name = match self.next() {
            Some(CssToken::At(n)) => n.clone(),
            _ => String::new(),
        };

        // Collect prelude until opening brace
        let mut prelude = String::new();
        while let Some(token) = self.peek() {
            if matches!(token, CssToken::OpenBrace) {
                break;
            }
            prelude.push_str(&format!("{:?}", token));
            self.next();
        }

        self.expect(&CssToken::OpenBrace);

        let mut content = Vec::new();
        let mut depth = 1;

        while depth > 0 && self.peek().is_some() {
            match self.peek() {
                Some(CssToken::OpenBrace) => {
                    depth += 1;
                    self.next();
                }
                Some(CssToken::CloseBrace) => {
                    depth -= 1;
                    if depth == 0 {
                        self.next();
                        break;
                    }
                    self.next();
                }
                _ => {
                    if let Some(rule) = self.parse_rule() {
                        content.push(CssItem::Rule(rule));
                    } else {
                        self.next();
                    }
                }
            }
        }

        CssItem::AtRule {
            name,
            prelude,
            content,
        }
    }

    fn parse_rule(&mut self) -> Option<Rule> {
        let selector = self.parse_selector()?;

        self.expect(&CssToken::OpenBrace);

        let declarations = self.parse_declarations();

        self.expect(&CssToken::CloseBrace);

        Some(Rule {
            selector,
            declarations,
        })
    }

    fn parse_selector(&mut self) -> Option<Selector> {
        let mut selector = self.parse_simple_selector()?;

        loop {
            match self.peek() {
                Some(CssToken::Greater) => {
                    self.next();
                    let right = self.parse_simple_selector()?;
                    selector = Selector::Child(Box::new(selector), Box::new(right));
                }
                Some(CssToken::Plus) => {
                    self.next();
                    let right = self.parse_simple_selector()?;
                    selector = Selector::Adjacent(Box::new(selector), Box::new(right));
                }
                Some(CssToken::Tilde) => {
                    self.next();
                    let right = self.parse_simple_selector()?;
                    selector = Selector::GeneralSibling(Box::new(selector), Box::new(right));
                }
                Some(CssToken::Comma) | Some(CssToken::OpenBrace) => break,
                Some(_) => {
                    // Descendant selector (implicit whitespace)
                    if let Some(right) = self.parse_simple_selector() {
                        selector = Selector::Descendant(Box::new(selector), Box::new(right));
                    } else {
                        break;
                    }
                }
                None => break,
            }
        }

        Some(selector)
    }

    fn parse_simple_selector(&mut self) -> Option<Selector> {
        match self.peek() {
            Some(CssToken::Asterisk) => {
                self.next();
                Some(Selector::Universal)
            }
            Some(CssToken::Hash(id)) => {
                let id = id.clone();
                self.next();
                Some(Selector::Id(id))
            }
            Some(CssToken::Dot(class)) => {
                let class = class.clone();
                self.next();
                Some(Selector::Class(class))
            }
            Some(CssToken::Ident(tag)) => {
                let tag = tag.clone();
                self.next();
                Some(Selector::Element(tag))
            }
            Some(CssToken::Colon) => {
                self.next();
                match self.next() {
                    Some(CssToken::Ident(name)) => {
                        let name = name.clone();
                        Some(Selector::PseudoClass(name))
                    }
                    _ => None,
                }
            }
            Some(CssToken::DoubleColon) => {
                self.next();
                match self.next() {
                    Some(CssToken::Ident(name)) => {
                        let name = name.clone();
                        Some(Selector::PseudoElement(name))
                    }
                    _ => None,
                }
            }
            Some(CssToken::OpenBracket) => {
                self.next();
                let name = match self.next() {
                    Some(CssToken::Ident(n)) => n.clone(),
                    _ => return None,
                };

                let (operator, value) = match self.peek() {
                    Some(CssToken::Equals) => {
                        self.next();
                        let val = match self.next() {
                            Some(CssToken::String(s)) => s.clone(),
                            Some(CssToken::Ident(s)) => s.clone(),
                            _ => String::new(),
                        };
                        (Some(AttrOperator::Exact), Some(val))
                    }
                    Some(CssToken::Includes) => {
                        self.next();
                        let val = match self.next() {
                            Some(CssToken::String(s)) => s.clone(),
                            Some(CssToken::Ident(s)) => s.clone(),
                            _ => String::new(),
                        };
                        (Some(AttrOperator::Contains), Some(val))
                    }
                    Some(CssToken::DashMatch) => {
                        self.next();
                        let val = match self.next() {
                            Some(CssToken::String(s)) => s.clone(),
                            Some(CssToken::Ident(s)) => s.clone(),
                            _ => String::new(),
                        };
                        (Some(AttrOperator::Dash), Some(val))
                    }
                    Some(CssToken::SubstringMatch) => {
                        self.next();
                        let val = match self.next() {
                            Some(CssToken::String(s)) => s.clone(),
                            Some(CssToken::Ident(s)) => s.clone(),
                            _ => String::new(),
                        };
                        (Some(AttrOperator::Substring), Some(val))
                    }
                    Some(CssToken::PrefixMatch) => {
                        self.next();
                        let val = match self.next() {
                            Some(CssToken::String(s)) => s.clone(),
                            Some(CssToken::Ident(s)) => s.clone(),
                            _ => String::new(),
                        };
                        (Some(AttrOperator::Prefix), Some(val))
                    }
                    Some(CssToken::SuffixMatch) => {
                        self.next();
                        let val = match self.next() {
                            Some(CssToken::String(s)) => s.clone(),
                            Some(CssToken::Ident(s)) => s.clone(),
                            _ => String::new(),
                        };
                        (Some(AttrOperator::Suffix), Some(val))
                    }
                    _ => (None, None),
                };

                self.expect(&CssToken::CloseBracket);

                Some(Selector::Attribute {
                    name,
                    operator,
                    value,
                })
            }
            _ => None,
        }
    }

    fn parse_declarations(&mut self) -> Vec<Declaration> {
        let mut declarations = Vec::new();

        while !matches!(self.peek(), Some(CssToken::CloseBrace) | None) {
            if let Some(decl) = self.parse_declaration() {
                declarations.push(decl);
            } else {
                self.next();
            }
        }

        declarations
    }

    fn parse_declaration(&mut self) -> Option<Declaration> {
        let property = match self.next() {
            Some(CssToken::Ident(prop)) => prop.clone().to_lowercase(),
            _ => return None,
        };

        self.expect(&CssToken::Colon);

        let value = self.parse_property_value();

        let important = if matches!(self.peek(), Some(CssToken::Ident(s)) if s.to_lowercase() == "important") {
            self.next();
            true
        } else {
            false
        };

        self.expect(&CssToken::Semicolon);

        Some(Declaration {
            property,
            value,
            important,
        })
    }

    fn parse_property_value(&mut self) -> String {
        let mut value = String::new();
        let mut depth = 0;

        while let Some(token) = self.peek() {
            match token {
                CssToken::Semicolon if depth == 0 => break,
                CssToken::CloseBrace if depth == 0 => break,
                CssToken::OpenParen | CssToken::OpenBracket => {
                    depth += 1;
                    value.push_str(&format!("{:?}", token));
                    self.next();
                }
                CssToken::CloseParen | CssToken::CloseBracket => {
                    depth -= 1;
                    value.push_str(&format!("{:?}", token));
                    self.next();
                }
                _ => {
                    value.push_str(&self.token_to_string(token));
                    self.next();
                }
            }
        }

        value.trim().to_string()
    }

    fn token_to_string(&self, token: &CssToken) -> String {
        match token {
            CssToken::Ident(s) => s.clone(),
            CssToken::Number(n) => n.to_string(),
            CssToken::Percentage(p) => format!("{}%", p),
            CssToken::Dimension { value, unit } => format!("{}{}", value, unit),
            CssToken::Color(c) => c.clone(),
            CssToken::String(s) => format!("\"{}\"", s),
            CssToken::Url(u) => format!("url({})", u),
            CssToken::Function(f) => format!("{}(", f),
            CssToken::Colon => ":".to_string(),
            CssToken::DoubleColon => "::".to_string(),
            CssToken::Comma => ",".to_string(),
            _ => String::new(),
        }
    }
}
