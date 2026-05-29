#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    StringLit(String),
    Number(i32),
    Punctuation(char), // ., :, ,
    // v1.4: Math operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    LParen,
    RParen,
    LessThan,
    GreaterThan,
    EqualEqual,
    NotEqual,
    LessEqual,
    GreaterEqual,
    Indent,
    Dedent,
    Comment(String),
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
            if self.pos < self.chars.len() && (self.chars[self.pos] == '\n' || (self.chars[self.pos] == '\r' && self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '\n')) {
                if self.chars[self.pos] == '\r' { self.pos += 1; }
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

        // v1.4: Math operators and parentheses
        match ch {
            '(' => { self.pos += 1; return Some(Token::LParen); }
            ')' => { self.pos += 1; return Some(Token::RParen); }
            '+' => { self.pos += 1; return Some(Token::Plus); }
            '*' => { self.pos += 1; return Some(Token::Star); }
            '/' => { self.pos += 1; return Some(Token::Slash); }
            '%' => { self.pos += 1; return Some(Token::Percent); }
            '-' => {
                // Don't consume '-' if followed by a digit (negative number handled in parser)
                self.pos += 1;
                return Some(Token::Minus);
            }
            '<' => {
                self.pos += 1;
                if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                    self.pos += 1;
                    return Some(Token::LessEqual);
                }
                return Some(Token::LessThan);
            }
            '>' => {
                self.pos += 1;
                if self.pos < self.chars.len() && self.chars[self.pos] == '=' {
                    self.pos += 1;
                    return Some(Token::GreaterEqual);
                }
                return Some(Token::GreaterThan);
            }
            '=' => {
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '=' {
                    self.pos += 2;
                    return Some(Token::EqualEqual);
                }
                // Single '=' not used in Syl, skip
                self.pos += 1;
                return self.next_token();
            }
            '!' => {
                if self.pos + 1 < self.chars.len() && self.chars[self.pos + 1] == '=' {
                    self.pos += 2;
                    return Some(Token::NotEqual);
                }
                self.pos += 1;
                return self.next_token();
            }
            _ => {}
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
                    // Handle namespaced calls like 'sys.exit'
                    s.push(c);
                    self.pos += 1;
                } else {
                    break;
                }
            }

            if s == "Note" && self.pos < self.chars.len() && self.chars[self.pos] == ':' {
                self.pos += 1; // Consume ':'
                let mut comment = String::new();
                while self.pos < self.chars.len() && self.chars[self.pos] != '\n' {
                    comment.push(self.chars[self.pos]);
                    self.pos += 1;
                }
                return Some(Token::Comment(comment.trim().to_string()));
            }

            // The lexer keeps "The", "A", etc., as tokens. The parser contextually ignores them.
            let fillers = ["please", "now"];
            for f in &fillers {
                if s.eq_ignore_ascii_case(f) {
                    return self.next_token();
                }
            }

            let mut final_s = s.clone();
            let keywords = ["behavior", "trigger", "self", "on", "wait", "sleep", "terminal", "box", "cursor", "milliseconds", "press", "connect", "database", "execute", "query", "current", "time", "date", "attempt", "fails", "succeeds", "json", "http", "request", "hits", "dictionary", "listen", "when", "reply", "verify", "set", "print", "give", "bring", "define", "enforce", "check", "run", "return", "when", "otherwise", "the", "a", "an", "to", "in", "as", "action", "called", "taking", "and", "that", "is", "not", "empty", "or", "crash", "with", "background", "it", "write", "read", "list", "files", "for", "each", "file", "if", "ends", "move", "create", "folder", "external", "from", "execute", "key", "pressed", "item", "make", "increase", "by", "space", "entity", "of", "download", "joined", "square", "root", "power", "random", "between", "size", "project", "named", "version", "target", "standalone", "executable", "delta", "time", "mouse", "draw", "rectangle", "circle", "at", "width", "height", "radius", "colored", "words", "repeat", "while", "begin", "clear", "end"];
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

        self.pos += 1; // Fallback
        self.next_token()
    }
}
