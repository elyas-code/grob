#[derive(Debug, Clone)]
pub enum Token {
    StartTag {
        name: String,
        attributes: Vec<(String, String)>,
    },
    EndTag {
        name: String,
    },
    Text(String),
}

pub struct Tokenizer {
    input: Vec<char>,
    pos: usize,
}

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

    pub fn next_token(&mut self) -> Option<Token> {
        let c = self.peek()?;

        if c == '<' {
            self.next();

            if self.peek() == Some('/') {
                self.next();
                let name = self.consume_while(|c| c.is_alphanumeric());
                self.consume_whitespace();
                self.next();
                return Some(Token::EndTag { name });
            }

            let name = self.consume_while(|c| c.is_alphanumeric());
            let mut attributes = Vec::new();

            loop {
                self.consume_whitespace();
                match self.peek() {
                    Some('>') => {
                        self.next();
                        break;
                    }
                    Some(_) => {
                        let key = self.consume_while(|c| c.is_alphanumeric());
                        self.consume_whitespace();
                        self.next();
                        self.consume_whitespace();
                        self.next();
                        let value = self.consume_while(|c| c != '"');
                        self.next();
                        attributes.push((key, value));
                    }
                    None => break,
                }
            }

            Some(Token::StartTag { name, attributes })
        } else {
            let text = self.consume_while(|c| c != '<');
            Some(Token::Text(text))
        }
    }
}
