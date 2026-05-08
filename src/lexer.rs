#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    StringLit(String),
    Number(i32),
    Punctuation(char), // ., :, ,
    Indent,
    Dedent,
    EOF,
}

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    indent_stack: Vec<usize>,
    pending_tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            chars: input.chars().collect(),
            pos: 0,
            indent_stack: vec![0],
            pending_tokens: Vec::new(),
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(tok) = self.next_token() {
            if tok == Token::EOF {
                break;
            }
            tokens.push(tok);
        }
        
        // Emit dedents for remaining indentation
        while self.indent_stack.len() > 1 {
            self.indent_stack.pop();
            tokens.push(Token::Dedent);
        }
        
        tokens.push(Token::EOF);
        tokens
    }

    fn next_token(&mut self) -> Option<Token> {
        if !self.pending_tokens.is_empty() {
            return Some(self.pending_tokens.remove(0));
        }

        if self.pos >= self.chars.len() {
            return Some(Token::EOF);
        }

        let ch = self.chars[self.pos];

        // Handle whitespace and newlines (indentation)
        if ch == '\n' {
            self.pos += 1;
            let mut spaces = 0;
            while self.pos < self.chars.len() && self.chars[self.pos] == ' ' {
                spaces += 1;
                self.pos += 1;
            }
            
            // Skip empty lines
            if self.pos < self.chars.len() && self.chars[self.pos] == '\n' {
                return self.next_token();
            }

            let current_indent = *self.indent_stack.last().unwrap();
            if spaces > current_indent {
                self.indent_stack.push(spaces);
                return Some(Token::Indent);
            } else if spaces < current_indent {
                while self.indent_stack.len() > 1 && *self.indent_stack.last().unwrap() > spaces {
                    self.indent_stack.pop();
                    self.pending_tokens.push(Token::Dedent);
                }
                return Some(self.pending_tokens.remove(0));
            } else {
                return self.next_token();
            }
        }

        if ch.is_whitespace() {
            self.pos += 1;
            return self.next_token();
        }

        if ch == '"' {
            self.pos += 1;
            let mut s = String::new();
            while self.pos < self.chars.len() && self.chars[self.pos] != '"' {
                s.push(self.chars[self.pos]);
                self.pos += 1;
            }
            self.pos += 1; // consume closing quote
            return Some(Token::StringLit(s));
        }

        if ch.is_ascii_digit() {
            let mut s = String::new();
            while self.pos < self.chars.len() && self.chars[self.pos].is_ascii_digit() {
                s.push(self.chars[self.pos]);
                self.pos += 1;
            }
            return Some(Token::Number(s.parse().unwrap()));
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut s = String::new();
            while self.pos < self.chars.len() {
                let c = self.chars[self.pos];
                if c.is_alphanumeric() || c == '_' {
                    s.push(c);
                    self.pos += 1;
                } else if c == '.' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1].is_alphabetic() {
                    s.push(c);
                    self.pos += 1;
                } else {
                    break;
                }
            }
            if s.eq_ignore_ascii_case("Note") && self.pos < self.chars.len() && self.chars[self.pos] == ':' {
                self.pos += 1;
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    self.pos += 1;
                }
                return self.next_token();
            }

            let fillers = ["the", "a", "an", "please", "now"];
            for f in &fillers {
                if s.eq_ignore_ascii_case(f) {
                    return self.next_token();
                }
            }

            let mut final_s = s.clone();
            let keywords = ["Set", "Print", "Give", "Bring", "Define", "Enforce", "Check", "Run", "Return", "When", "Otherwise", "to", "in", "as", "action", "called", "taking", "and", "that", "is", "not", "empty", "or", "crash", "with", "background", "it", "Write", "Read", "List", "files", "For", "each", "file", "If", "ends", "Move", "Create", "folder", "External", "from", "Execute", "key", "pressed", "item", "Make", "Increase", "by", "Space"];
            for kw in &keywords {
                if s.eq_ignore_ascii_case(kw) {
                    final_s = kw.to_string();
                    break;
                }
            }
            return Some(Token::Word(final_s));
        }

        if ch == '.' || ch == ':' || ch == ',' {
            self.pos += 1;
            return Some(Token::Punctuation(ch));
        }

        self.pos += 1; // Fallback, shouldn't hit for well-formed Syl code
        self.next_token()
    }
}
