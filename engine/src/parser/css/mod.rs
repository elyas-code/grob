#[derive(Debug, Clone)]
pub enum CssToken {
    Ident(String),
    OpenBrace,
    CloseBrace,
    Colon,
    Semicolon,
    Hash(String),
    Dot(String),
    String(String),
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

    fn next_char(&mut self) -> Option<char> {
        if self.pos >= self.input.len() { None } else { 
            let c = self.input[self.pos]; 
            self.pos += 1; 
            Some(c)
        }
    }

    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    pub fn next_token(&mut self) -> Option<CssToken> {
        while let Some(c) = self.next_char() {
            match c {
                '{' => return Some(CssToken::OpenBrace),
                '}' => return Some(CssToken::CloseBrace),
                ':' => return Some(CssToken::Colon),
                ';' => return Some(CssToken::Semicolon),
                '#' => {
                    let mut ident = String::new();
                    while let Some(nc) = self.peek() {
                        if nc.is_alphanumeric() { ident.push(nc); self.pos +=1 } else { break; }
                    }
                    return Some(CssToken::Hash(ident));
                },
                '.' => {
                    let mut ident = String::new();
                    while let Some(nc) = self.peek() {
                        if nc.is_alphanumeric() { ident.push(nc); self.pos +=1 } else { break; }
                    }
                    return Some(CssToken::Dot(ident));
                },
                c if c.is_whitespace() => continue,
                c => {
                    let mut ident = String::new();
                    ident.push(c);
                    while let Some(nc) = self.peek() {
                        if nc.is_alphanumeric() || nc=='-' { ident.push(nc); self.pos+=1 } else { break; }
                    }
                    return Some(CssToken::Ident(ident));
                }
            }
        }
        None
    }
}
