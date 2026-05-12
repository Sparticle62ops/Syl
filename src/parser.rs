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
                target: "executable".to_string(),
            }
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

    pub fn parse(&mut self) -> Result<Vec<Statement>, String> {
        self.parse_metadata()?;
        let mut statements = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::EOF || *tok == Token::Dedent {
                break;
            }
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_metadata(&mut self) -> Result<(), String> {
        while let Some(Token::Word(w)) = self.peek() {
            if w == "The" {
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
                    "Bring" => self.parse_bring(),
                    "Define" => self.parse_define(),
                    "Enforce" => self.parse_enforce(),
                    "Set" => self.parse_set(),
                    "Check" => self.parse_check(),
                    "Run" => self.parse_run(),
                    "Return" => self.parse_return(),
                    "Print" => self.parse_print(),
                    "Download" => self.parse_download(),
                    "Give" => self.parse_give(),
                    "Write" => self.parse_write(),
                    "Read" => self.parse_read(),
                    "List" => self.parse_list(),
                    "Move" => self.parse_move(),
                    "Create" => self.parse_create(),
                    "If" => self.parse_if(),
                    "For" => self.parse_for_each(),
                    "External" => self.parse_external(),
                    "Execute" => self.parse_execute(),
                    "Add" => self.parse_add_to_list(),
                    "open" => self.parse_open_window(),
                    "Repeat" => self.parse_repeat_while(),
                    "Begin" => self.parse_begin_drawing(),
                    "Clear" => self.parse_clear_background(),
                    "Draw" => self.parse_draw_text(),
                    "End" => self.parse_end_drawing(),
                    "Make" => self.parse_make_item(),
                    "Increase" => self.parse_increase(),
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

        let mut body = Vec::new();
        if filename == "sys" {
            println!("[ PARSE ] sys (virtual)");
            self.imported_modules.insert(filename.clone());
            return Ok(Statement::Import { filename, alias, body });
        }

        if !self.imported_modules.contains(&filename) {
            println!("[ PARSE ] {}", filename);
            self.imported_modules.insert(filename.clone());
            
            let file_path = self.base_path.join(&filename);
            let source = match fs::read_to_string(&file_path) {
                Ok(s) => s,
                Err(e) => return Err(format!("Failed to import module {}: {}", filename, e)),
            };
            
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            
            let mut child_parser = Parser::new(tokens, self.base_path, self.imported_modules);
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
        self.expect_word("Define")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "an" || w == "a" {
                self.advance();
            }
        }
        
        match self.peek() {
            Some(Token::Word(w)) if w == "Entity" => self.parse_define_entity(),
            _ => self.parse_define_action(),
        }
    }

    fn parse_define_entity(&mut self) -> Result<Statement, String> {
        self.expect_word("Entity")?;
        self.expect_word("called")?;
        let name = self.expect_ident()?;
        self.expect_punct(':')?;
        
        match self.advance() {
            Some(Token::Indent) => {}
            other => return Err(self.error_msg(&format!("I understood you are defining the Entity '{}', but I was expecting a block of fields (Indent), got {:?}", name, other))),
        }
        
        let mut fields = Vec::new();
        while let Some(tok) = self.peek() {
            if *tok == Token::Dedent { break; }
            self.expect_word("Set")?;
            let field_name = self.expect_ident()?;
            self.expect_word("to")?;
            let value = self.parse_expr()?;
            self.expect_punct('.')?;
            fields.push((field_name, value));
        }
        
        self.advance(); // consume Dedent
        Ok(Statement::DefineEntity { name, fields })
    }

    fn parse_set(&mut self) -> Result<Statement, String> {
        self.expect_word("Set")?;
        let left = self.parse_expr()?;
        
        if let Some(Token::Word(w)) = self.peek() {
            if w != "to" {
                if let Expr::Identifier(id) = &left {
                    return Err(self.error_msg(&format!("I understood you are trying to Set '{}', but I was expecting the word 'to' followed by a value.", id)));
                }
            }
        }
        
        self.expect_word("to")?;
        let value = self.parse_expr()?;
        self.expect_punct('.')?;
        
        match left {
            Expr::Identifier(name) => Ok(Statement::Assign { name, value }),
            Expr::GetField { field_name, entity_instance } => Ok(Statement::SetField { field_name, entity_instance, value }),
            other => Err(self.error_msg(&format!("I understood you are trying to Set something, but {:?} is not something I can assign a value to.", other))),
        }
    }

    fn parse_download(&mut self) -> Result<Statement, String> {
        self.expect_word("Download")?;
        self.expect_word("from")?;
        let url = self.parse_expr()?;
        self.expect_word("as")?;
        let target = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::Download { url, target })
    }

    fn parse_enforce(&mut self) -> Result<Statement, String> {
        self.expect_word("Enforce")?;
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

    fn parse_set(&mut self) -> Result<Statement, String> {
        self.expect_word("Set")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "the" {
                self.advance();
            }
        }
        let name = self.expect_ident()?;
        self.expect_word("to")?;
        if let Some(Token::Word(w)) = self.peek() {
            if w == "item" {
                self.advance();
                let index = self.parse_expr()?;
                self.expect_word("in")?;
                let list = self.expect_ident()?;
                self.expect_punct('.')?;
                return Ok(Statement::GetItem { identifier: name, index, list });
            }
        }
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

    fn parse_write(&mut self) -> Result<Statement, String> {
        self.expect_word("Write")?;
        let data = self.parse_expr()?;
        self.expect_word("to")?;
        let path = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Write { data, path })
    }

    fn parse_read(&mut self) -> Result<Statement, String> {
        self.expect_word("Read")?;
        let path = self.parse_expr()?;
        self.expect_word("as")?;
        let identifier = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::Read { path, identifier })
    }

    fn parse_list(&mut self) -> Result<Statement, String> {
        self.expect_word("List")?;
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
        self.expect_word("Move")?;
        let path = self.parse_expr()?;
        self.expect_word("to")?;
        let destination = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Move { path, destination })
    }

    fn parse_create(&mut self) -> Result<Statement, String> {
        self.expect_word("Create")?;
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
        Ok(Statement::DefineAction { name, args, body })
    }

    fn parse_add_to_list(&mut self) -> Result<Statement, String> {
        self.expect_word("Add")?;
        let item = self.parse_expr()?;
        self.expect_word("to")?;
        let list_name = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::AddToList { item, list_name })
    }

    fn parse_open_window(&mut self) -> Result<Statement, String> {
        self.expect_word("open")?;
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
        self.expect_word("Repeat")?;
        self.expect_word("while")?;
        self.expect_word("window")?;
        self.expect_word("is")?;
        self.expect_word("not")?;
        self.expect_word("closing")?;
        self.expect_punct(':')?;
        let body = self.parse_block()?;
        Ok(Statement::WhileNot { condition_action: "ui_WindowShouldClose".into(), body })
    }

    fn parse_begin_drawing(&mut self) -> Result<Statement, String> {
        self.expect_word("Begin")?;
        self.expect_word("drawing")?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_BeginDrawing".into(), args: vec![] })
    }

    fn parse_clear_background(&mut self) -> Result<Statement, String> {
        self.expect_word("Clear")?;
        self.expect_word("background")?;
        self.expect_word("to")?;
        let color = self.expect_ident()?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_ClearBackground".into(), args: vec![crate::ast::Expr::Identifier(color)] })
    }

    fn parse_draw_text(&mut self) -> Result<Statement, String> {
        self.expect_word("Draw")?;
        self.expect_word("text")?;
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

    fn parse_end_drawing(&mut self) -> Result<Statement, String> {
        self.expect_word("End")?;
        self.expect_word("drawing")?;
        self.expect_punct('.')?;
        Ok(Statement::CallAction { name: "ui_EndDrawing".into(), args: vec![] })
    }

    fn parse_make_item(&mut self) -> Result<Statement, String> {
        self.expect_word("Make")?;
        self.expect_word("item")?;
        let index = self.parse_expr()?;
        self.expect_word("in")?;
        let list = self.expect_ident()?;
        let value = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::SetItem { index, list, value })
    }

    fn parse_increase(&mut self) -> Result<Statement, String> {
        self.expect_word("Increase")?;
        let name = self.expect_ident()?;
        self.expect_word("by")?;
        let amount = self.parse_expr()?;
        self.expect_punct('.')?;
        Ok(Statement::Increase { name, amount })
    }

    fn parse_if(&mut self) -> Result<Statement, String> {
        self.expect_word("If")?;
        let ident = self.expect_ident()?;
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
            }
        }
        Err("Unsupported If format".into())
    }

    fn parse_for_each(&mut self) -> Result<Statement, String> {
        self.expect_word("For")?;
        self.expect_word("each")?;
        let item = self.expect_ident()?;
        self.expect_word("in")?;
        let collection = self.expect_ident()?;
        self.expect_punct(':')?;
        let body = self.parse_block()?;
        Ok(Statement::ForEach { item, collection, body })
    }

    fn parse_external(&mut self) -> Result<Statement, String> {
        self.expect_word("External")?;
        let namespace = self.expect_ident()?;
        let action = self.expect_ident()?;
        self.expect_word("from")?;
        let lib_path = match self.parse_expr()? {
            Expr::StringLit(s) => s,
            _ => return Err("Expected string literal for library path".into()),
        };
        self.expect_punct('.')?;
        Ok(Statement::External { namespace, action, lib_path })
    }

    fn parse_execute(&mut self) -> Result<Statement, String> {
        self.expect_word("Execute")?;
        let namespace = self.expect_ident()?;
        let action = self.expect_ident()?;
        self.expect_word("with")?;
        let mut args = Vec::new();
        args.push(self.parse_expr()?);
        while let Some(Token::Word(w)) = self.peek() {
            if w == "and" {
                self.advance();
                args.push(self.parse_expr()?);
            } else {
                break;
            }
        }
        self.expect_punct('.')?;
        Ok(Statement::Execute { namespace, action, args })
    }

    // Check the user_role:
    fn parse_check(&mut self) -> Result<Statement, String> {
        self.expect_word("Check")?;
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
        let mut left = self.parse_primary_expr()?;
        
        while let Some(Token::Word(w)) = self.peek() {
            match w.as_str() {
                "of" => {
                    self.advance();
                    let instance = self.expect_ident()?;
                    if let Expr::Identifier(field) = left {
                        left = Expr::GetField { field_name: field, entity_instance: instance };
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
                        if w2 == "the" {
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
        Ok(left)
    }

    fn parse_primary_expr(&mut self) -> Result<Expr, String> {
        let tok = self.advance().cloned();
        match tok {
            Some(Token::StringLit(s)) => Ok(Expr::StringLit(s)),
            Some(Token::Number(n)) => Ok(Expr::Number(n)),
            Some(Token::Word(w)) => {
                match w.as_str() {
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
                                _ => {}
                            }
                        }
                    }
                    "a" => {
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
    }

    fn error_msg(&self, msg: &str) -> String {
        format!("\n[ SYL GRAMMAR ] {}\n", msg)
    }
}
