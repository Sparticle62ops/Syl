use crate::ast::*;
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        self.pos += 1;
        tok
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        match self.advance() {
            Some(Token::Word(w)) if w == expected => Ok(()),
            other => Err(format!("Expected word '{}', got {:?}", expected, other)),
        }
    }

    fn expect_ident(&mut self) -> Result<String, String> {
        match self.advance() {
            Some(Token::Word(w)) => Ok(w.clone()),
            other => Err(format!("Expected identifier, got {:?}", other)),
        }
    }

    fn expect_punct(&mut self, expected: char) -> Result<(), String> {
        match self.advance() {
            Some(Token::Punctuation(p)) if *p == expected => Ok(()),
            other => Err(format!("Expected punctuation '{}', got {:?}", expected, other)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::EOF || *tok == Token::Dedent {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        match self.advance() {
            Some(Token::Indent) => {}
            other => return Err(format!("Expected Indent, got {:?}", other)),
        }
        let statements = self.parse()?;
        match self.advance() {
            Some(Token::Dedent) => {}
            other => return Err(format!("Expected Dedent, got {:?}", other)),
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let first = self.peek().cloned();
        match first {
            Some(Token::Word(w)) => {
                match w.as_str() {
                    "Bring" => self.parse_bring(),
                    "Define" => self.parse_define(),
                    "Enforce" => self.parse_enforce(),
                    "Set" => self.parse_set(),
                    "Check" => self.parse_check(),
                    "Run" => self.parse_run(),
                    "Return" => self.parse_return(),
                    "Print" => self.parse_print(),
                    "Give" => self.parse_give(),
                    _ => Err(format!("Unknown statement starting with '{}'", w)),
                }
            }
            other => Err(format!("Expected statement, got {:?}", other)),
        }
    }

    // Bring in "auth.syl" as auth.
    fn parse_bring(&mut self) -> Result<Statement, String> {
        self.expect_word("Bring")?;
        self.expect_word("in")?;
        let filename = match self.advance() {
            Some(Token::StringLit(s)) => s.clone(),
            other => return Err(format!("Expected string, got {:?}", other)),
        };
        self.expect_word("as")?;
        let alias = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::Import { filename, alias })
    }

    // Define an action called process_login taking username and password:
    fn parse_define(&mut self) -> Result<Statement, String> {
        self.expect_word("Define")?;
        self.expect_word("an")?;
        self.expect_word("action")?;
        self.expect_word("called")?;
        let name = self.expect_ident()?;
        
        let mut args = Vec::new();
        if let Some(Token::Word(w)) = self.peek() {
            if w == "taking" {
                self.advance();
                args.push(self.expect_ident()?);
                while let Some(Token::Word(w)) = self.peek() {
                    if w == "and" {
                        self.advance();
                        args.push(self.expect_ident()?);
                    } else {
                        break;
                    }
                }
            }
        }
        self.expect_punct(':')?;
        let body = self.parse_block()?;
        Ok(Statement::DefineAction { name, args, body })
    }

    // Enforce that the username is not empty, or crash with "Missing username".
    fn parse_enforce(&mut self) -> Result<Statement, String> {
        self.expect_word("Enforce")?;
        self.expect_word("that")?;
        self.expect_word("the")?;
        let name = self.expect_ident()?;
        self.expect_word("is")?;
        self.expect_word("not")?;
        self.expect_word("empty")?;
        self.expect_punct(',')?;
        self.expect_word("or")?;
        self.expect_word("crash")?;
        self.expect_word("with")?;
        let crash_msg = match self.advance() {
            Some(Token::StringLit(s)) => s.clone(),
            _ => return Err("Expected string".into()),
        };
        self.expect_punct('.')?;
        Ok(Statement::Enforce { name, condition: "not empty".into(), crash_msg })
    }

    // Set the user_role to auth.get_role taking the username.
    fn parse_set(&mut self) -> Result<Statement, String> {
        self.expect_word("Set")?;
        self.expect_word("the")?;
        let name = self.expect_ident()?;
        self.expect_word("to")?;
        let value = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Assign { name, value })
    }

    // Print the message. OR Print "Hello".
    fn parse_print(&mut self) -> Result<Statement, String> {
        self.expect_word("Print")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
        let value = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Print { value })
    }

    // Give the message to the void_action.
    fn parse_give(&mut self) -> Result<Statement, String> {
        self.expect_word("Give")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
        let src = self.expect_ident()?;
        self.expect_word("to")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
        let dst = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::Give { src, dst })
    }

    // Check the user_role:
    fn parse_check(&mut self) -> Result<Statement, String> {
        self.expect_word("Check")?;
        self.expect_word("the")?;
        let name = self.expect_ident()?;
        self.expect_punct(':')?;

        match self.advance() {
            Some(Token::Indent) => {}
            other => return Err(format!("Expected Indent for Check, got {:?}", other)),
        }

        let mut cases = Vec::new();
        while let Some(Token::Word(w)) = self.peek() {
            if w == "When" {
                self.advance();
                self.expect_word("it")?;
                self.expect_word("is")?;
                let val = self.parse_expr()?;
                self.expect_punct(':')?;
                let body = self.parse_block()?;
                cases.push(MatchCase { value: val, body });
            } else if w == "Otherwise" {
                self.advance();
                self.expect_punct(':')?;
                let body = self.parse_block()?;
                cases.push(MatchCase { value: Expr::Identifier("Otherwise".into()), body }); // special marker
            } else {
                break;
            }
        }
        
        match self.advance() {
            Some(Token::Dedent) => {}
            other => return Err(format!("Expected Dedent for Check, got {:?}", other)),
        }

        Ok(Statement::Match { name, cases })
    }

    // Run auth.log_admin_login in the background.
    fn parse_run(&mut self) -> Result<Statement, String> {
        self.expect_word("Run")?;
        let call = self.parse_expr()?;
        self.expect_word("in")?;
        self.expect_word("the")?;
        self.expect_word("background")?;
        self.expect_punct('.')?;
        Ok(Statement::RunBackground { action_call: call })
    }

    // Return "Welcome Admin".
    fn parse_return(&mut self) -> Result<Statement, String> {
        self.expect_word("Return")?;
        let val = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Return { value: val })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let tok = self.advance().cloned();
        match tok {
            Some(Token::StringLit(s)) => Ok(Expr::StringLit(s)),
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Word(w)) => {
                let mut is_call = w.contains('.');
                let mut args = Vec::new();
                
                if let Some(Token::Word(next_w)) = self.peek() {
                    if next_w == "taking" {
                        is_call = true;
                        self.advance();
                        
                        if let Some(Token::Word(w_the)) = self.peek() {
                            if w_the == "the" {
                                self.advance();
                            }
                        }
                        
                        args.push(self.parse_expr()?);
                        
                        while let Some(Token::Word(w_and)) = self.peek() {
                            if w_and == "and" {
                                self.advance();
                                args.push(self.parse_expr()?);
                            } else {
                                break;
                            }
                        }
                    }
                }
                
                if is_call {
                    Ok(Expr::Call { action: w, args })
                } else {
                    Ok(Expr::Identifier(w))
                }
            }
            _ => Err("Invalid expression".into()),
        }
    }
}
