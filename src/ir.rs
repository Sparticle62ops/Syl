use crate::ast::*;
use std::collections::HashMap;

pub struct IRGenerator {
    var_map: HashMap<String, usize>,
    next_reg: usize,
    pub instructions: Vec<String>,
}

impl IRGenerator {
    pub fn new() -> Self {
        IRGenerator {
            var_map: HashMap::new(),
            next_reg: 0,
            instructions: vec!["SYLA".to_string()],
        }
    }

    fn b52(val: usize) -> char {
        let val = val % 52;
        if val < 26 {
            (b'A' + val as u8) as char
        } else {
            (b'a' + (val - 26) as u8) as char
        }
    }

    fn get_reg(&mut self, name: &str) -> usize {
        if let Some(&reg) = self.var_map.get(name) {
            reg
        } else {
            let reg = self.next_reg;
            self.next_reg += 1;
            self.var_map.insert(name.to_string(), reg);
            reg
        }
    }

    pub fn generate(&mut self, ast: &[Statement]) {
        for stmt in ast {
            self.gen_statement(stmt);
        }
    }

    fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Assign { name, value } => {
                let reg = self.get_reg(name);
                self.gen_expr(value, reg);
            }
            Statement::Create { name, value, .. } => {
                let reg = self.get_reg(name);
                self.gen_expr(value, reg);
            }
            Statement::Mutate { name, value } => {
                let reg = self.get_reg(name);
                self.gen_expr(value, reg);
            }
            Statement::Copy { src, dst } => {
                let src_reg = self.get_reg(src);
                let dst_reg = self.get_reg(dst);
                self.emit('C', 'O', src_reg, dst_reg);
            }
            Statement::Lend { src, dst } => {
                let src_reg = self.get_reg(src);
                let dst_reg = self.get_reg(dst);
                self.emit('L', 'h', src_reg, dst_reg);
            }
            Statement::Give { src, dst } => {
                let src_reg = self.get_reg(src);
                let dst_reg = self.get_reg(dst);
                self.emit('g', 'o', src_reg, dst_reg);
            }
            Statement::Discard { name } => {
                let reg = self.get_reg(name);
                self.emit('d', 'o', reg, 0);
            }
            Statement::IfElse { condition_var, then_branch, else_branch, .. } => {
                let reg = self.get_reg(condition_var);
                self.emit('I', 'O', reg, 0);
                self.generate(then_branch);
                if let Some(elb) = else_branch {
                    self.emit('i', 'O', reg, 0);
                    self.generate(elb);
                }
            }
            Statement::Match { name, cases } => {
                let reg = self.get_reg(name);
                self.emit('H', 'O', reg, 0);
                for case in cases {
                    self.generate(&case.body);
                }
            }
            Statement::ForEach { item, collection, body } => {
                let item_reg = self.get_reg(item);
                let col_reg = self.get_reg(collection);
                self.emit('F', 'O', col_reg, item_reg);
                self.generate(body);
            }
            Statement::Repeat { body, .. } => {
                self.emit('R', 'O', 0, 0);
                self.generate(body);
            }
            Statement::Import { alias, body, .. } => {
                let reg = self.get_reg(alias);
                self.emit('B', 'O', reg, 0);
                self.generate(body);
            }
            Statement::DefineAction { name, body, .. } => {
                let reg = self.get_reg(name);
                self.emit('D', 'O', reg, 0);
                self.generate(body);
            }
            Statement::ExportAction { action } => {
                self.gen_statement(action);
                self.emit('X', 'O', 0, 0);
            }
            Statement::RunBackground { action_call } => {
                self.gen_expr(action_call, 0);
                self.emit('U', 'O', 0, 0);
            }
            Statement::Listen { .. } => {
                self.emit('S', 'O', 0, 0);
            }
            Statement::Wait { action_call } => {
                self.gen_expr(action_call, 0);
                self.emit('W', 'O', 0, 0);
            }
            Statement::Return { value } => {
                self.gen_expr(value, 0);
                self.emit('T', 'O', 0, 0);
            }
            Statement::Enforce { name, .. } => {
                let reg = self.get_reg(name);
                self.emit('N', 'O', reg, 0);
            }
            Statement::Print { value } => {
                self.gen_expr(value, 0);
                self.emit('P', 'O', 0, 0);
            }
            Statement::Write { data, path } => {
                self.gen_expr(data, 0);
                self.gen_expr(path, 1);
                self.emit('W', 'O', 0, 1);
            }
            Statement::Read { path, identifier } => {
                self.gen_expr(path, 0);
                let reg = self.get_reg(identifier);
                self.emit('R', 'O', 0, reg);
            }
            Statement::List { path, identifier } => {
                self.gen_expr(path, 0);
                let reg = self.get_reg(identifier);
                self.emit('L', 'O', 0, reg);
            }
            Statement::Move { path, destination } => {
                self.gen_expr(path, 0);
                self.gen_expr(destination, 1);
                self.emit('m', 'O', 0, 1);
            }
            Statement::CreateFolder { path } => {
                self.gen_expr(path, 0);
                self.emit('K', 'O', 0, 0);
            }
            Statement::IfEndsWith { filename, body, .. } => {
                let reg = self.get_reg(filename);
                self.emit('I', 'O', reg, 0);
                self.generate(body);
            }
            Statement::External { .. } => {
                self.emit('E', 'O', 0, 0);
            }
            Statement::Execute { .. } => {
                self.emit('E', 'O', 0, 0);
            }
            Statement::CreateList { name, items } => {
                let reg = self.get_reg(name);
                self.emit('A', 'O', reg, 0);
                for item in items {
                    self.gen_expr(item, reg);
                }
            }
            Statement::AddToList { list_name, item } => {
                let reg = self.get_reg(list_name);
                self.gen_expr(item, reg);
            }
            Statement::CallAction { name, args } => {
                let reg = self.get_reg(name);
                for (i, arg) in args.iter().enumerate() {
                    self.gen_expr(arg, i);
                }
                self.emit('E', 'O', reg, 0);
            }
            Statement::WhileNot { body, .. } => {
                self.emit('R', 'O', 0, 0);
                self.generate(body);
            }
            Statement::IfKeyPressed { key, body } => {
                let reg = self.get_reg(key);
                self.emit('I', 'O', reg, 0);
                self.generate(body);
            }
            Statement::GetItem { identifier, list, index } => {
                let list_reg = self.get_reg(list);
                let id_reg = self.get_reg(identifier);
                self.gen_expr(index, id_reg);
                self.emit('I', 'O', list_reg, id_reg);
            }
            Statement::SetItem { list, index, value } => {
                let list_reg = self.get_reg(list);
                self.gen_expr(index, 0);
                self.gen_expr(value, 1);
                self.emit('I', 'O', list_reg, 0);
            }
            Statement::Increase { name, amount } => {
                let reg = self.get_reg(name);
                self.gen_expr(amount, reg);
                self.emit('A', 'O', reg, reg);
            }
            Statement::DefineEntity { name, .. } => {
                let reg = self.get_reg(name);
                self.emit('D', 'O', reg, 0);
            }
            Statement::CreateEntity { name, .. } => {
                let reg = self.get_reg(name);
                self.emit('A', 'O', reg, 0);
            }
            Statement::SetField { entity_instance, .. } => {
                let reg = self.get_reg(entity_instance);
                self.emit('A', 'O', reg, 0);
            }
            Statement::Download { target, url } => {
                self.gen_expr(url, 0);
                let reg = self.get_reg(target);
                self.emit('A', 'O', 0, reg);
            }
            Statement::ListWords { identifier, source } => {
                self.gen_expr(source, 0);
                let reg = self.get_reg(identifier);
                self.emit('L', 'O', 0, reg);
            }
            _ => {
                // Remaining statement types emit a no-op
                self.emit('N', 'O', 0, 0);
            }
        }
    }

    fn gen_expr(&mut self, expr: &Expr, target_reg: usize) {
        match expr {
            Expr::Call { .. } => {
                self.emit('E', 'O', 0, target_reg);
            }
            Expr::Identifier(ident) => {
                let reg = self.get_reg(ident);
                self.emit('A', 'O', reg, target_reg);
            }
            Expr::StringLit(_) => {
                self.emit('A', 'O', 0, target_reg);
            }
            Expr::Number(_) => {
                self.emit('A', 'O', 0, target_reg);
            }
            Expr::GetField { entity_instance, .. } => {
                let reg = self.get_reg(entity_instance);
                self.emit('A', 'O', reg, target_reg);
            }
            Expr::Join { left, right } => {
                self.gen_expr(left, target_reg);
                self.gen_expr(right, target_reg);
                self.emit('J', 'O', target_reg, target_reg);
            }
            Expr::Sqrt { value } => {
                self.gen_expr(value, target_reg);
                self.emit('M', 'O', target_reg, 0);
            }
            Expr::Pow { base, exponent } => {
                self.gen_expr(base, target_reg);
                self.gen_expr(exponent, target_reg);
                self.emit('M', 'O', target_reg, target_reg);
            }
            Expr::Random { min, max } => {
                self.gen_expr(min, target_reg);
                self.gen_expr(max, target_reg);
                self.emit('M', 'O', target_reg, 0);
            }
            Expr::FileSize { path } => {
                self.gen_expr(path, target_reg);
                self.emit('A', 'O', 0, target_reg);
            }
            Expr::BinaryOp { left, right, .. } => {
                self.gen_expr(left, target_reg);
                self.gen_expr(right, target_reg);
                self.emit('M', 'O', target_reg, target_reg);
            }
            Expr::UnaryNeg { value } => {
                self.gen_expr(value, target_reg);
                self.emit('M', 'O', target_reg, 0);
            }
            Expr::DeltaTime | Expr::MouseX | Expr::MouseY => {
                self.emit('E', 'O', 0, target_reg);
            }
        }
    }

    fn emit(&mut self, op: char, typ: char, arg1: usize, arg2: usize) {
        let ins = format!("{}{}{}{}", op, typ, Self::b52(arg1), Self::b52(arg2));
        self.instructions.push(ins);
    }
}

