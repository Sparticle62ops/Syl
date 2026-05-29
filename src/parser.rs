use crate::ast::*;
use crate::lexer::{Token, Lexer};
use std::path::Path;
use std::collections::HashSet;
use std::fs;

pub struct Parser<'a> {
    tokens: Vec<Token>,
    pos: usize,
    base_path: &'a Path,
    imported_modules: &'a mut HashSet<String>,
    pub metadata: ProjectMetadata,
    pub is_sandboxed: bool,
    last_comment: Option<String>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: Vec<Token>, base_path: &'a Path, imported_modules: &'a mut HashSet<String>) -> Self {
        Parser { 
            tokens, 
            pos: 0, 
            base_path, 
            imported_modules,
            metadata: ProjectMetadata {
                name: "Untitled".to_string(),
                version: "0.1.0".to_string(),
                target: "c".to_string(),
            },
            is_sandboxed: false,
            last_comment: None,
        }
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

    fn expect_token(&mut self, expected: Token) -> Result<(), String> {
        match self.advance() {
            Some(t) if *t == expected => Ok(()),
            other => Err(format!("Expected {:?}, got {:?}", expected, other)),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        self.parse_metadata()?;
        let mut statements = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::EOF || *tok == Token::Dedent {
                break;
            }
            if let Some(Token::Comment(c)) = self.peek() {
                self.last_comment = Some(c.clone());
                self.advance();
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_metadata(&mut self) -> Result<(), String> {
        while let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
                match self.peek() {
                    Some(Token::Word(w2)) if w2 == "project" => {
                        self.advance();
                        self.expect_word("is")?;
                        self.expect_word("named")?;
                        if let Some(Token::StringLit(name)) = self.advance() {
                            self.metadata.name = name.clone();
                        }
                        self.expect_punct('.')?;
                    }
                    Some(Token::Word(w2)) if w2 == "version" => {
                        self.advance();
                        self.expect_word("is")?;
                        if let Some(Token::StringLit(ver)) = self.advance() {
                            self.metadata.version = ver.clone();
                        }
                        self.expect_punct('.')?;
                    }
                    Some(Token::Word(w2)) if w2 == "target" => {
                        self.advance();
                        self.expect_word("is")?;
                        self.expect_word("a")?;
                        self.expect_word("standalone")?;
                        self.expect_word("executable")?;
                        self.metadata.target = "executable".to_string();
                        self.expect_punct('.')?;
                    }
                    _ => {
                        self.pos -= 1; // Backtrack "The"
                        break;
                    }
                }
            } else {
                break;
            }
        }
        Ok(())
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
                    // Skip decorative words at statement level
                    "please" | "now" | "the" | "a" | "an" => { self.advance(); return self.parse_statement(); }
                    "bring" => self.parse_bring(),
                    "define" => self.parse_define(),
                    "trigger" => self.parse_trigger(),
                    "enforce" => self.parse_enforce(),
                    "set" => self.parse_set(),
                    "check" => self.parse_check(),
                    "run" => self.parse_run(),
                    "return" => self.parse_return(),
                    "print" => self.parse_print(),
                    "download" => self.parse_download(),
                    "give" => self.parse_give(),
                    "write" => self.parse_write(),
                    "read" => self.parse_read(),
                    "list" => self.parse_list(),
                    "move" => self.parse_move(),
                    "create" => self.parse_create(),
                    "if" => self.parse_if(),
                    "for" => self.parse_for_each(),
                    "external" => self.parse_external(),
                    "execute" => self.parse_execute_or_query(),
                    "add" => self.parse_add_to_list(),
                    "open" => self.parse_open_window(),
                    "repeat" => self.parse_repeat_while(),
                    "begin" => self.parse_begin_drawing(),
                    "clear" => self.parse_clear_background(),
                    "draw" => self.parse_draw(),
                    "end" => self.parse_end_drawing(),
                    "make" => self.parse_make_item(),
                    "increase" => self.parse_increase(),
                    "verify" => self.parse_verify(),
                    "listen" => self.parse_listen_http(),
                    "when" => self.parse_http_route(),
                    "reply" => self.parse_http_reply(),
                    "attempt" => self.parse_attempt(),
                    "connect" => self.parse_connect_db(),
                    "wait" => self.parse_wait(),
                    "sleep" => self.parse_sleep(),
                    _ => Err(self.error_msg(&format!("I don't understand the statement starting with '{}'. Check your spelling or keyword.", w))),
                }
            }
            other => { println!("ERROR AT POS: {}", self.pos); return Err(self.error_msg(&format!("Expected a statement, but found {:?}.", other))); }
        }
    }
    // Bring in "auth.syl" as auth.
    fn parse_bring(&mut self) -> Result<Statement, String> {
        self.expect_word("bring")?;
        self.expect_word("in")?;
        
        let mut is_package = false;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "package" {
                self.advance();
                is_package = true;
            }
        }
        
        let tok = self.advance().cloned();
        let filename = match tok {
            Some(Token::StringLit(s)) => s,
            other => return Err(self.error_msg(&format!("Expected a module name in quotes after 'Bring in', but found {:?}.", other))),
        };
        self.expect_word("as")?;
        let alias = self.expect_ident()?;
        self.expect_punct('.')?;

        let mut body = Vec::new();

        // Virtual modules (built-in, no file needed)
        let virtual_modules = ["sys", "core", "ui", "net", "math", "db", "time"];
        if virtual_modules.contains(&filename.as_str()) {
            println!("[ PARSE ] {} (virtual module)", filename);
            self.imported_modules.insert(filename.clone());
            return Ok(Statement::Import { filename, alias, body });
        }

        if !self.imported_modules.contains(&filename) {
            self.imported_modules.insert(filename.clone());

            // Resolve the module file path with fallback chain:
            // 1. Local directory (relative to the importing file)
            // 2. SYL_LIB_PATH environment variable
            // 3. Executable-relative ../lib/ directory
            let syl_filename = if filename.ends_with(".syl") || filename.ends_with(".syx") {
                filename.clone()
            } else {
                format!("{}.syl", filename)
            };

            let local_path = self.base_path.join(&syl_filename);
            let resolved_path = if local_path.exists() {
                println!("[ PARSE ] {} (local)", syl_filename);
                local_path
            } else if let Ok(lib_path) = std::env::var("SYL_LIB_PATH") {
                let global_path = std::path::PathBuf::from(&lib_path).join(&syl_filename);
                if global_path.exists() {
                    println!("[ PARSE ] {} (SYL_LIB_PATH: {})", syl_filename, lib_path);
                    global_path
                } else {
                    return Err(self.error_msg(&format!(
                        "I could not find the module '{}'. I looked in:\n  1. {}\n  2. {}",
                        filename, local_path.display(), global_path.display()
                    )));
                }
            } else {
                // Fallback: look relative to the syl executable
                let exe_lib = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|d| d.join("../lib").join(&syl_filename)));
                if let Some(ref exe_path) = exe_lib {
                    if exe_path.exists() {
                        println!("[ PARSE ] {} (SDK lib/)", syl_filename);
                        exe_path.clone()
                    } else {
                        return Err(self.error_msg(&format!(
                            "I could not find the module '{}'. I looked in:\n  1. {}\n  Set SYL_LIB_PATH to your Syl standard library directory.",
                            filename, local_path.display()
                        )));
                    }
                } else {
                    return Err(self.error_msg(&format!(
                        "I could not find the module '{}' in the local directory.", filename
                    )));
                }
            };

            let source = match fs::read_to_string(&resolved_path) {
                Ok(s) => s,
                Err(e) => return Err(self.error_msg(&format!("Failed to read module '{}': {}", filename, e))),
            };
            
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            
            let mut child_parser = Parser::new(tokens, self.base_path, self.imported_modules);
            if is_package || filename.ends_with(".syx") || syl_filename.ends_with(".syx") {
                child_parser.is_sandboxed = true;
            }
            let mut ast = child_parser.parse()?;
            
            // Namespace prefixing for exported actions
            for stmt in &mut ast {
                if let Statement::DefineAction { name, .. } = stmt {
                    *name = format!("{}_{}", alias, name);
                }
            }
            body = ast;
        }

        Ok(Statement::Import { filename, alias, body })
    }

    fn parse_define(&mut self) -> Result<Statement, String> {
        self.expect_word("define")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "an" || w == "a" {
                self.advance();
            }
        }
        
        match self.peek() {
            Some(Token::Word(w)) if w == "entity" => self.parse_define_entity(),
            Some(Token::Word(w)) if w == "behavior" => self.parse_define_behavior(),
            _ => self.parse_define_action(),
        }
    }

    fn parse_define_entity(&mut self) -> Result<Statement, String> {
        self.expect_word("entity")?;
        self.expect_word("called")?;
        let name = self.expect_ident()?;
        self.expect_punct(':')?;
        
        let tok = self.advance().cloned();
        match tok {
            Some(Token::Indent) => {}
            other => return Err(self.error_msg(&format!("I understood you are defining the Entity '{}', but I was expecting a block of fields (Indent), got {:?}", name, other))),
        }
        
        let mut fields = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::Dedent { break; }
            self.expect_word("set")?;
            let field_name = self.expect_ident()?;
            self.expect_word("to")?;
            let value = self.parse_expr()?;
            self.expect_punct('.')?;
            fields.push((field_name, value));
        }
        
        self.expect_token(Token::Dedent)?;
        Ok(Statement::DefineEntity { name, fields, doc: self.last_comment.take() })
    }

    fn parse_set(&mut self) -> Result<Statement, String> {
        self.expect_word("set")?;
        
        // Handle "Set key "X" in dict to value."
        if let Some(Token::Word(w)) = self.peek() {
            if w == "key" {
                self.advance();
                let key = self.parse_expr()?;
                self.expect_word("in")?;
                let dict = self.expect_ident()?;
                self.expect_word("to")?;
                let value = self.parse_expr()?;
                self.expect_punct('.')?;
                return Ok(Statement::SetDictKey { dict, key, value });
            }
        }

        let left = self.parse_expr()?;
        
        // Conversational error: missing 'to'
        if let Some(Token::Word(w)) = self.peek() {
            if w != "to" {
                if let Expr::Identifier(id) = &left {
                    return Err(self.error_msg(&format!("I understood you are trying to Set '{}', but I was expecting the word 'to' followed by a value.", id)));
                }
            }
        }
        
        self.expect_word("to")?;
        
        // Check for GetItem pattern: "Set X to item N in list."
        if let Some(Token::Word(w)) = self.peek() {
            if w == "item" {
                if let Expr::Identifier(name) = left {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect_word("in")?;
                    let list = self.expect_ident()?;
                    self.expect_punct('.')?;
                    return Ok(Statement::GetItem { identifier: name, index, list });
                }
            }
        }
        
        // Check for GetDictKey pattern: "Set X to key "K" in D."
        if let Some(Token::Word(w)) = self.peek() {
            if w == "key" {
                if let Expr::Identifier(name) = left {
                    self.advance();
                    let key = self.parse_expr()?;
                    self.expect_word("in")?;
                    let dict = self.expect_ident()?;
                    self.expect_punct('.')?;
                    return Ok(Statement::Assign { name, value: Expr::GetDictKey { dict, key: Box::new(key) } });
                }
            }
        }
        
        let value = self.parse_expr()?;
        self.expect_punct('.')?;
        
        match left {
            Expr::Identifier(name) => Ok(Statement::Assign { name, value }),
            Expr::GetField { field_name, entity_instance } => Ok(Statement::SetField { field_name, entity_instance, value }),
            Expr::SelfField { field_name } => Ok(Statement::SetField { field_name, entity_instance: "self".to_string(), value }),
            other => Err(self.error_msg(&format!("I understood you are trying to Set something, but {:?} is not something I can assign a value to.", other))),
        }
    }

    fn parse_download(&mut self) -> Result<Statement, String> {
        self.expect_word("download")?;
        self.expect_word("from")?;
        let url = self.parse_expr()?;
        self.expect_word("as")?;
        let target = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::Download { url, target })
    }

    fn parse_enforce(&mut self) -> Result<Statement, String> {
        self.expect_word("enforce")?;
        self.expect_word("that")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
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




    // Print the message. OR Print "Hello".
    fn parse_print(&mut self) -> Result<Statement, String> {
        self.expect_word("print")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
        let value = self.parse_expr()?;
        
        if let Some(Token::Word(w)) = self.peek() {
            if w == "colored" {
                self.advance();
                let color_tok = self.advance().cloned();
                let color = match color_tok {
                    Some(Token::StringLit(c)) => c,
                    _ => return Err(self.error_msg("Expected string color name after 'colored'")),
                };
                self.expect_punct('.')?;
                return Ok(Statement::PrintColored { text: value, color });
            }
        }
        
        self.expect_punct('.')?;
        Ok(Statement::Print { value })
    }

    // Give the message to the void_action.
    fn parse_give(&mut self) -> Result<Statement, String> {
        self.expect_word("give")?;
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

    fn parse_write(&mut self) -> Result<Statement, String> {
        self.expect_word("write")?;
        let data = self.parse_expr()?;
        self.expect_word("to")?;
        let path = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Write { data, path })
    }

    fn parse_read(&mut self) -> Result<Statement, String> {
        self.expect_word("read")?;
        let path = self.parse_expr()?;
        self.expect_word("as")?;
        let identifier = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::Read { path, identifier })
    }

    fn parse_list(&mut self) -> Result<Statement, String> {
        self.expect_word("list")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "words" {
                self.advance();
                self.expect_word("in")?;
                let source = self.parse_expr()?;
                self.expect_word("as")?;
                let identifier = self.expect_ident()?;
                self.expect_punct('.')?;
                return Ok(Statement::ListWords { source, identifier });
            }
            if w == "files" {
                self.advance();
                self.expect_word("in")?;
                let path = self.parse_expr()?;
                self.expect_word("as")?;
                let identifier = self.expect_ident()?;
                self.expect_punct('.')?;
                return Ok(Statement::List { path, identifier });
            }
        }
        Err(self.error_msg("I was expecting 'words' or 'files' after 'List'."))
    }

    fn parse_move(&mut self) -> Result<Statement, String> {
        self.expect_word("move")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "cursor" {
                self.advance();
                self.expect_word("to")?;
                let x = self.parse_expr()?;
                self.expect_punct(',')?;
                let y = self.parse_expr()?;
                self.expect_punct('.')?;
                return Ok(Statement::MoveCursor { x, y });
            }
        }
        let path = self.parse_expr()?;
        self.expect_word("to")?;
        let destination = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Move { path, destination })
    }

    fn parse_create(&mut self) -> Result<Statement, String> {
        self.expect_word("create")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "an" || w == "a" {
                self.advance();
            }
        }

        let type_name = self.expect_ident()?;
        if type_name == "folder" {
            let path = self.parse_expr()?;
            self.expect_punct('.')?;
            return Ok(Statement::CreateFolder { path });
        } else if type_name == "Dictionary" {
            self.expect_word("called")?;
            let name = self.expect_ident()?;
            self.expect_punct('.')?;
            return Ok(Statement::CreateDictionary { name });
        } else if type_name == "List" {
            self.expect_word("called")?;
            let name = self.expect_ident()?;
            self.expect_word("containing")?;
            let mut items = Vec::new();
            items.push(self.parse_expr()?);
            while let Some(Token::Word(w)) = self.peek() {
                if w == "and" || w == "," {
                    self.advance();
                    items.push(self.parse_expr()?);
                } else {
                    break;
                }
            }
            self.expect_punct('.')?;
            return Ok(Statement::CreateList { name, items });
        } else {
            self.expect_word("called")?;
            let name = self.expect_ident()?;
            self.expect_punct('.')?;
            return Ok(Statement::CreateEntity { entity_type: type_name, name });
        }
    }

    fn parse_define_action(&mut self) -> Result<Statement, String> {
        self.expect_word("action")?;
        self.expect_word("called")?;
        let name = self.expect_ident()?;
        
        let mut args = Vec::new();
        if let Some(Token::Word(w)) = self.peek() {
            if w == "taking" {
                self.advance();
                args.push(self.expect_ident()?);
                while let Some(Token::Word(w_and)) = self.peek() {
                    if w_and == "and" {
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
        Ok(Statement::DefineAction { name, args, body, doc: self.last_comment.take() })
    }

    fn parse_define_behavior(&mut self) -> Result<Statement, String> {
        self.expect_word("behavior")?;
        let action_name = self.expect_ident()?;
        self.expect_word("for")?;
        let entity_name = self.expect_ident()?;
        
        let mut args = Vec::new();
        if let Some(Token::Word(w)) = self.peek() {
            if w == "taking" {
                self.advance();
                args.push(self.expect_ident()?);
                while let Some(Token::Word(w_and)) = self.peek() {
                    if w_and == "and" {
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
        Ok(Statement::DefineBehavior { entity_name, action_name, args, body, doc: self.last_comment.take() })
    }

    fn parse_trigger(&mut self) -> Result<Statement, String> {
        self.expect_word("trigger")?;
        let action_name = self.expect_ident()?;
        self.expect_word("on")?;
        let instance = self.expect_ident()?;
        
        let mut args = Vec::new();
        if let Some(Token::Word(w)) = self.peek() {
            if w == "taking" {
                self.advance();
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
        
        self.expect_punct('.')?;
        Ok(Statement::TriggerBehavior { action_name, instance, args })
    }

    fn parse_add_to_list(&mut self) -> Result<Statement, String> {
        self.expect_word("add")?;
        let item = self.parse_expr()?;
        self.expect_word("to")?;
        let list_name = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::AddToList { item, list_name })
    }

    fn parse_open_window(&mut self) -> Result<Statement, String> {
        self.expect_word("open")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "a" || w == "an" || w == "the" {
                self.advance();
            }
        }
        self.expect_word("window")?;
        self.expect_word("with")?;
        self.expect_word("width")?;
        let width = self.parse_expr()?;
        self.expect_word("and")?;
        self.expect_word("height")?;
        let height = self.parse_expr()?;
        self.expect_word("titled")?;
        let title = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_InitWindow".into(), args: vec![width, height, title] })
    }

    fn parse_repeat_while(&mut self) -> Result<Statement, String> {
        self.expect_word("repeat")?;
        self.expect_word("while")?;
        
        let saved_pos = self.pos;
        let mut is_window_not_closing = false;
        
        // Skip decorative 'the' if present
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
        
        if let Some(Token::Word(w2)) = self.peek() {
            if w2 == "window" {
                self.advance();
                if let Some(Token::Word(w3)) = self.peek() {
                    if w3 == "is" {
                        self.advance();
                        if let Some(Token::Word(w4)) = self.peek() {
                            if w4 == "not" {
                                self.advance();
                                if let Some(Token::Word(w5)) = self.peek() {
                                    if w5 == "closing" {
                                        self.advance();
                                        is_window_not_closing = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        if is_window_not_closing {
            self.expect_punct(':')?;
            let body = self.parse_block()?;
            return Ok(Statement::WhileNot { condition_action: "WindowShouldClose".into(), body });
        } else {
            self.pos = saved_pos;
            let condition = self.parse_conditional_expr()?;
            self.expect_punct(':')?;
            let body = self.parse_block()?;
            return Ok(Statement::While { condition, body });
        }
    }


    fn parse_begin_drawing(&mut self) -> Result<Statement, String> {
        self.expect_word("begin")?;
        self.expect_word("drawing")?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_BeginDrawing".into(), args: vec![] })
    }

    fn parse_clear_background(&mut self) -> Result<Statement, String> {
        self.expect_word("clear")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "a" || w == "an" || w == "the" {
                self.advance();
            }
        }
        
        if let Some(Token::Word(w)) = self.peek() {
            if w == "terminal" {
                self.advance();
                self.expect_punct('.')?;
                return Ok(Statement::ClearTerminal);
            }
        }

        self.expect_word("background")?;
        self.expect_word("to")?;
        let color = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_ClearBackground".into(), args: vec![color] })
    }

    fn parse_draw(&mut self) -> Result<Statement, String> {
        self.expect_word("draw")?;
        match self.peek() {
            Some(Token::Word(w)) if w == "terminal" => {
                self.advance();
                self.expect_word("box")?;
                self.expect_word("at")?;
                let x = self.parse_expr()?;
                self.expect_punct(',')?;
                let y = self.parse_expr()?;
                self.expect_word("with")?;
                self.expect_word("width")?;
                let w = self.parse_expr()?;
                self.expect_word("and")?;
                self.expect_word("height")?;
                let h = self.parse_expr()?;
                self.expect_punct('.')?;
                Ok(Statement::DrawTerminalBox { x, y, w, h })
            }
            Some(Token::Word(w)) if w == "circle" => self.parse_draw_circle(),
            Some(Token::Word(w)) if w == "rectangle" => self.parse_draw_rectangle(),
            _ => self.parse_draw_text_no_keyword(),
        }
    }

    fn parse_draw_text_no_keyword(&mut self) -> Result<Statement, String> {
        if let Some(Token::Word(w)) = self.peek() {
            if w == "text" { self.advance(); }
        }
        let mut text_args = Vec::new();
        text_args.push(self.parse_expr()?);
        while let Some(Token::Word(w)) = self.peek() {
            if w == "and" {
                self.advance();
                text_args.push(self.parse_expr()?);
            } else {
                break;
            }
        }
        self.expect_word("at")?;
        self.expect_word("position")?;
        let x = self.parse_expr()?;
        if let Some(Token::Punctuation(p)) = self.peek() {
            if *p == ',' {
                self.advance();
            }
        }
        let y = self.parse_expr()?;
        self.expect_punct('.')?;
        let mut final_args = text_args;
        final_args.push(x);
        final_args.push(y);
        Ok(Statement::CallAction { name: "ui_DrawText".into(), args: final_args })
    }

    fn parse_draw_circle(&mut self) -> Result<Statement, String> {
        self.expect_word("circle")?;
        self.expect_word("at")?;
        let x = self.parse_expr()?;
        if let Some(Token::Punctuation(p)) = self.peek() {
            if *p == ',' { self.advance(); }
        }
        let y = self.parse_expr()?;
        self.expect_word("with")?;
        self.expect_word("radius")?;
        let radius = self.parse_expr()?;
        self.expect_word("colored")?;
        let color = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_DrawCircle".into(), args: vec![x, y, radius, color] })
    }

    fn parse_draw_rectangle(&mut self) -> Result<Statement, String> {
        self.expect_word("rectangle")?;
        self.expect_word("at")?;
        let x = self.parse_expr()?;
        if let Some(Token::Punctuation(p)) = self.peek() {
            if *p == ',' { self.advance(); }
        }
        let y = self.parse_expr()?;
        self.expect_word("with")?;
        self.expect_word("width")?;
        let w = self.parse_expr()?;
        self.expect_word("and")?;
        self.expect_word("height")?;
        let h = self.parse_expr()?;
        self.expect_word("colored")?;
        let color = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_DrawRectangle".into(), args: vec![x, y, w, h, color] })
    }

    fn parse_end_drawing(&mut self) -> Result<Statement, String> {
        self.expect_word("end")?;
        self.expect_word("drawing")?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_EndDrawing".into(), args: vec![] })
    }

    fn parse_make_item(&mut self) -> Result<Statement, String> {
        self.expect_word("make")?;
        self.expect_word("item")?;
        let index = self.parse_expr()?;
        self.expect_word("in")?;
        let list = self.expect_ident()?;
        let value = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::SetItem { index, list, value })
    }

    fn parse_increase(&mut self) -> Result<Statement, String> {
        self.expect_word("increase")?;
        let name = self.expect_ident()?;
        self.expect_word("by")?;
        let amount = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Increase { name, amount })
    }

    fn parse_verify(&mut self) -> Result<Statement, String> {
        self.expect_word("verify")?;
        self.expect_word("that")?;
        let left = self.parse_expr()?;
        self.expect_word("is")?;
        let right = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Verify { left, right })
    }

    fn parse_listen_http(&mut self) -> Result<Statement, String> {
        self.expect_word("listen")?;
        self.expect_word("for")?;
        self.expect_word("http")?;
        self.expect_word("on")?;
        self.expect_word("port")?;
        let port = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::ListenHttp { port })
    }

    fn parse_http_route(&mut self) -> Result<Statement, String> {
        self.expect_word("when")?;
        let mut is_request = false;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "a" {
                self.advance();
                self.expect_word("request")?;
                self.expect_word("hits")?;
                is_request = true;
            }
        }
        if !is_request {
            return Err(self.error_msg("Expected 'When a request hits ...'"));
        }
        let path = self.parse_expr()?;
        self.expect_punct(':')?;
        let body = self.parse_block()?;
        Ok(Statement::HttpRequestRoute { path, body })
    }

    fn parse_http_reply(&mut self) -> Result<Statement, String> {
        self.expect_word("reply")?;
        self.expect_word("with")?;
        let content = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::HttpReply { content })
    }

    fn parse_attempt(&mut self) -> Result<Statement, String> {
        self.expect_word("attempt")?;
        self.expect_word("to")?;
        let action = Box::new(self.parse_statement()?);
        Ok(Statement::Attempt { action })
    }

    fn error_msg(&self, msg: &str) -> String {
        format!("\n[ SYL GRAMMAR ] {}\n", msg)
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect_word("if")?;

        // v1.4: If (math_expr): — expression-based conditional
        if let Some(Token::LParen) = self.peek() {
            let condition = self.parse_conditional_expr()?;
            self.expect_punct(':')?;
            let then_branch = self.parse_block()?;
            let mut else_branch = None;
            if let Some(Token::Word(w)) = self.peek() {
                if w == "Otherwise" {
                    self.advance();
                    self.expect_punct(':')?;
                    else_branch = Some(self.parse_block()?);
                }
            }
            return Ok(Statement::IfExpr { condition, then_branch, else_branch });
        }

        let ident = self.expect_ident()?;
        if ident == "it" {
            if let Some(Token::Word(w2)) = self.peek() {
                if w2 == "fails" {
                    self.advance();
                    self.expect_punct(':')?;
                    let body = self.parse_block()?;
                    return Ok(Statement::IfFailed { body });
                } else if w2 == "succeeds" {
                    self.advance();
                    self.expect_punct(':')?;
                    let body = self.parse_block()?;
                    return Ok(Statement::IfSucceeded { body });
                }
            }
        }

        if let Some(Token::Word(w)) = self.peek() {
            if w == "key" {
                self.advance();
                self.expect_word("is")?;
                self.expect_word("pressed")?;
                self.expect_punct(':')?;
                let body = self.parse_block()?;
                return Ok(Statement::IfKeyPressed { key: ident, body });
            } else if w == "ends" {
                self.advance();
                self.expect_word("with")?;
                let extension = match self.parse_expr()? {
                    Expr::StringLit(s) => s,
                    _ => return Err("Expected string literal for extension".into()),
                };
                self.expect_punct(':')?;
                let body = self.parse_block()?;
                return Ok(Statement::IfEndsWith { filename: ident, extension, body });
            } else if w == "is" {
                self.advance();
                let val = self.parse_expr()?;
                self.expect_punct(':')?;
                let then_branch = self.parse_block()?;
                let mut else_branch = None;
                if let Some(Token::Word(ew)) = self.peek() {
                    if ew == "Otherwise" {
                        self.advance();
                        self.expect_punct(':')?;
                        else_branch = Some(self.parse_block()?);
                    }
                }
                return Ok(Statement::IfElse { condition_var: ident, condition_val: val, then_branch, else_branch });
            }
        }
        Err(self.error_msg("Unsupported If format. Use 'If (expr):' or 'If X is Y:' or 'If X key is pressed:'"))
    }

    fn parse_for_each(&mut self) -> Result<Statement, String> {
        self.expect_word("for")?;
        self.expect_word("each")?;
        let item = self.expect_ident()?;
        self.expect_word("in")?;
        let collection = self.expect_ident()?;
        self.expect_punct(':')?;
        let body = self.parse_block()?;
        Ok(Statement::ForEach { item, collection, body })
    }

    fn parse_external(&mut self) -> Result<Statement, String> {
        self.expect_word("external")?;
        let namespace = self.expect_ident()?;
        let action = self.expect_ident()?;
        self.expect_word("from")?;
        let lib_path = match self.parse_expr()? {
            Expr::StringLit(s) => s,
            _ => return Err("Expected string literal for library path".into()),
        };
        
        // Sandbox Guard Enforcer
        if self.is_sandboxed {
            let allowed_libs = ["raylib.dll", "math.h", "raylib"];
            if !allowed_libs.contains(&lib_path.as_str()) {
                return Err("[ HELIX FATAL ] Sandbox Violation: .syx packages are malware-proof and cannot execute raw OS commands.".into());
            }
        }
        
        self.expect_punct('.')?;
        Ok(Statement::External { namespace, action, lib_path })
    }

    fn parse_execute_or_query(&mut self) -> Result<Statement, String> {
        self.expect_word("execute")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "query" {
                self.advance();
                let query = self.parse_expr()?;
                self.expect_word("on")?;
                let db_identifier = self.expect_ident()?;
                
                let mut results_list = None;
                if let Some(Token::Word(w2)) = self.peek() {
                    if w2 == "and" {
                        self.advance();
                        self.expect_word("store")?;
                        self.expect_word("in")?;
                        results_list = Some(self.expect_ident()?);
                    }
                }
                
                self.expect_punct('.')?;
                return Ok(Statement::ExecuteQuery { query, db_identifier, results_list });
            }
        }
        
        // Fallback to normal execute
        let expr = self.parse_expr()?;
        self.expect_punct('.')?;
        if let Expr::Call { action, args } = expr {
            Ok(Statement::CallAction { name: action, args })
        } else {
            Err(self.error_msg("I was expecting an action call or 'query' after 'Execute'."))
        }
    }

    fn parse_connect_db(&mut self) -> Result<Statement, String> {
        self.expect_word("connect")?;
        self.expect_word("to")?;
        self.expect_word("database")?;
        let path = self.parse_expr()?;
        self.expect_word("as")?;
        let identifier = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::ConnectDB { path, identifier })
    }

    // Check the user_role:
    fn parse_check(&mut self) -> Result<Statement, String> {
        self.expect_word("check")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
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
        self.expect_word("run")?;
        let call = self.parse_expr()?;
        self.expect_word("in")?;
        self.expect_word("the")?;
        self.expect_word("background")?;
        self.expect_punct('.')?;
        Ok(Statement::RunBackground { action_call: call })
    }

    // Return "Welcome Admin".
    fn parse_return(&mut self) -> Result<Statement, String> {
        self.expect_word("return")?;
        let val = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Return { value: val })
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_primary_expr()?;
        
        while let Some(tok) = self.peek() {
            match tok {
                Token::Word(w) => {
                    match w.as_str() {
                        "of" => {
                            self.advance();
                            let instance = self.expect_ident()?;
                            if let Expr::Identifier(field) = left {
                                if instance == "self" {
                                    left = Expr::SelfField { field_name: field };
                                } else {
                                    left = Expr::GetField { field_name: field, entity_instance: instance };
                                }
                            } else {
                                return Err(self.error_msg("I understood you are using 'of', but I was expecting a property name before it."));
                            }
                        }
                        "joined" => {
                            self.advance();
                            self.expect_word("with")?;
                            let right = self.parse_expr()?;
                            left = Expr::Join { left: Box::new(left), right: Box::new(right) };
                        }
                        "to" => {
                            // Check for "to the power of"
                            let saved_pos = self.pos;
                            self.advance();
                            if let Some(Token::Word(w2)) = self.peek() {
                                if w2 == "the" || w2 == "The" {
                                    self.advance();
                                    if let Some(Token::Word(w3)) = self.peek() {
                                        if w3 == "power" {
                                            self.advance();
                                            self.expect_word("of")?;
                                            let right = self.parse_expr()?;
                                            left = Expr::Pow { base: Box::new(left), exponent: Box::new(right) };
                                            continue;
                                        }
                                    }
                                }
                            }
                            self.pos = saved_pos; // backtrack
                            break;
                        }
                        _ => break,
                    }
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_conditional_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_expr()?;
        loop {
            if let Some(Token::Word(w)) = self.peek() {
                match w.as_str() {
                    "and" => {
                        self.advance();
                        let right = self.parse_expr()?;
                        left = Expr::BinaryOp { op: BinOp::And, left: Box::new(left), right: Box::new(right) };
                    }
                    "or" => {
                        self.advance();
                        let right = self.parse_expr()?;
                        left = Expr::BinaryOp { op: BinOp::Or, left: Box::new(left), right: Box::new(right) };
                    }
                    "is" => {
                        self.advance();
                        let right = self.parse_expr()?;
                        left = Expr::BinaryOp { op: BinOp::Eq, left: Box::new(left), right: Box::new(right) };
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
        Ok(left)
    }

    // ── Math Block: Precedence-Climbing Expression Parser ──
    // Entered when we see `(` — parses full math with +, -, *, /, <, >, ==, etc.
    fn parse_math_expr(&mut self) -> Result<Expr, String> {
        self.parse_math_logical()
    }

    fn parse_math_logical(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_math_comparison()?;
        loop {
            let op = match self.peek() {
                Some(Token::Word(w)) if w == "and" => BinOp::And,
                Some(Token::Word(w)) if w == "or" => BinOp::Or,
                _ => break,
            };
            self.advance();
            let right = self.parse_math_comparison()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_math_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_math_additive()?;
        loop {
            let op = match self.peek() {
                Some(Token::LessThan) => BinOp::Lt,
                Some(Token::GreaterThan) => BinOp::Gt,
                Some(Token::EqualEqual) => BinOp::Eq,
                Some(Token::NotEqual) => BinOp::Neq,
                Some(Token::LessEqual) => BinOp::Lte,
                Some(Token::GreaterEqual) => BinOp::Gte,
                _ => break,
            };
            self.advance();
            let right = self.parse_math_additive()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_math_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_math_multiplicative()?;
        loop {
            let op = match self.peek() {
                Some(Token::Plus) => BinOp::Add,
                Some(Token::Minus) => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_math_multiplicative()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_math_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_math_unary()?;
        loop {
            let op = match self.peek() {
                Some(Token::Star) => BinOp::Mul,
                Some(Token::Slash) => BinOp::Div,
                Some(Token::Percent) => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_math_unary()?;
            left = Expr::BinaryOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_math_unary(&mut self) -> Result<Expr, String> {
        if let Some(Token::Minus) = self.peek() {
            self.advance();
            let val = self.parse_math_atom()?;
            return Ok(Expr::UnaryNeg { value: Box::new(val) });
        }
        self.parse_math_atom()
    }

    fn parse_math_atom(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::LParen) => {
                self.advance(); // consume (
                let expr = self.parse_math_expr()?;
                let next_tok = self.advance().cloned();
                match next_tok {
                    Some(Token::RParen) => Ok(expr),
                    other => Err(self.error_msg(&format!("Expected closing ')' in math block, found {:?}", other))),
                }
            }
            Some(Token::Number(_)) => {
                if let Some(Token::Number(n)) = self.advance().cloned() {
                    Ok(Expr::Number(n))
                } else {
                    unreachable!()
                }
            }
            Some(Token::Word(_)) => {
                if let Some(Token::Word(w)) = self.advance().cloned() {
                    // Check for function call inside math: `get_fib taking (n - 1)`
                    if let Some(Token::Word(nw)) = self.peek() {
                        if nw == "taking" {
                            self.advance();
                            let mut args = vec![self.parse_expr()?];
                            while let Some(Token::Word(wa)) = self.peek() {
                                if wa == "and" {
                                    self.advance();
                                    args.push(self.parse_expr()?);
                                } else {
                                    break;
                                }
                            }
                            return Ok(Expr::Call { action: w, args });
                        } else if nw == "of" {
                            self.advance(); // consume "of"
                            let instance = self.expect_ident()?;
                            if instance == "self" {
                                return Ok(Expr::SelfField { field_name: w });
                            } else {
                                return Ok(Expr::GetField { field_name: w, entity_instance: instance });
                            }
                        }
                    }
                    Ok(Expr::Identifier(w))
                } else {
                    unreachable!()
                }
            }
            other => Err(self.error_msg(&format!("Unexpected token in math expression: {:?}", other))),
        }
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, String> {
        let tok = self.advance().cloned();
        match tok {
            Some(Token::StringLit(s)) => Ok(Expr::StringLit(s)),
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            // v1.4: Parenthesized math block
            Some(Token::LParen) => {
                let expr = self.parse_math_expr()?;
                let next_tok = self.advance().cloned();
                match next_tok {
                    Some(Token::RParen) => Ok(expr),
                    other => Err(self.error_msg(&format!("Expected closing ')' in math block, found {:?}", other))),
                }
            }
            Some(Token::Word(w)) => {
                match w.as_str() {
                    // Handle capitalized articles that weren't stripped by lexer
                    "the" => {
                        if let Some(Token::Word(w2)) = self.peek() {
                            match w2.as_str() {
                                "square" => {
                                    self.advance();
                                    self.expect_word("root")?;
                                    self.expect_word("of")?;
                                    let val = self.parse_expr()?;
                                    return Ok(Expr::Sqrt { value: Box::new(val) });
                                }
                                "size" => {
                                    self.advance();
                                    self.expect_word("of")?;
                                    let val = self.parse_expr()?;
                                    return Ok(Expr::FileSize { path: Box::new(val) });
                                }
                                _ => {
                                    return self.parse_primary_expr();
                                }
                            }
                        }
                        return self.parse_primary_expr();
                    }
                    "a" | "an" => {
                        if let Some(Token::Word(w2)) = self.peek() {
                            if w2 == "random" {
                                self.advance();
                                self.expect_word("number")?;
                                self.expect_word("between")?;
                                let min = self.parse_expr()?;
                                self.expect_word("and")?;
                                let max = self.parse_expr()?;
                                return Ok(Expr::Random { min: Box::new(min), max: Box::new(max) });
                            }
                        }
                        // "a"/"an" are decorative, skip
                        return self.parse_primary_expr();
                    }
                    // v1.4: Raylib intrinsics
                    "delta" => {
                        self.expect_word("time")?;
                        return Ok(Expr::DeltaTime);
                    }
                    "mouse" => {
                        if let Some(Token::Word(axis)) = self.peek() {
                            match axis.as_str() {
                                "X" | "x" => { self.advance(); return Ok(Expr::MouseX); }
                                "Y" | "y" => { self.advance(); return Ok(Expr::MouseY); }
                                _ => {}
                            }
                        }
                        return Ok(Expr::Identifier(w));
                    }
                    "key" => {
                        let key_expr = self.parse_expr()?;
                        self.expect_word("in")?;
                        let dict = self.expect_ident()?;
                        return Ok(Expr::GetDictKey { dict, key: Box::new(key_expr) });
                    }
                    "json" => {
                        self.expect_word("from")?;
                        let dict = self.expect_ident()?;
                        return Ok(Expr::JsonFromDict { dict });
                    }
                    "dictionary" => {
                        self.expect_word("from")?;
                        self.expect_word("json")?;
                        let json = self.parse_expr()?;
                        return Ok(Expr::DictFromJson { json: Box::new(json) });
                    }
                    "current" => {
                        if let Some(Token::Word(w2)) = self.peek() {
                            if w2 == "time" {
                                self.advance();
                                return Ok(Expr::CurrentTime);
                            } else if w2 == "date" {
                                self.advance();
                                return Ok(Expr::CurrentDate);
                            }
                        }
                        return Ok(Expr::Identifier(w));
                    }
                    _ => {}
                }

                let mut is_call = w.contains('.');
                let mut args = Vec::new();
                
                if let Some(Token::Word(next_w)) = self.peek() {
                    if next_w == "taking" {
                        is_call = true;
                        self.advance();
                        
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
            other => Err(self.error_msg(&format!("I was expecting an expression here, but I found {:?}", other))),
        }
    }    fn parse_sleep(&mut self) -> Result<Statement, String> {
        self.expect_word("sleep")?;
        self.expect_word("for")?;
        let duration = self.parse_expr()?;
        self.expect_word("milliseconds")?;
        self.expect_punct('.')?;
        Ok(Statement::SleepMilliseconds { duration })
    }

    fn parse_wait(&mut self) -> Result<Statement, String> {
        self.expect_word("wait")?;
        self.expect_word("for")?;
        self.expect_word("key")?;
        self.expect_word("press")?;
        self.expect_word("as")?;
        let var = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::WaitForKeyPress { var })
    }
}


