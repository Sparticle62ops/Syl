use crate::ast::*;

pub struct CodeGen {
    pub c_code: String,
    indent_level: usize,
}

impl CodeGen {
    pub fn new() -> Self {
        let preamble = r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Syl v0.1 C Transpiler Preamble
typedef char* String;
"#;
        CodeGen {
            c_code: preamble.to_string(),
            indent_level: 0,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn push_line(&mut self, line: &str) {
        let ind = self.indent();
        self.c_code.push_str(&format!("{}{}\n", ind, line));
    }

    pub fn generate(&mut self, ast: &[Statement]) {
        for stmt in ast {
            self.gen_statement(stmt);
        }
    }

    fn gen_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Import { filename, alias } => {
                self.push_line(&format!("// Bring in {} as {}", filename, alias));
            }
            Statement::DefineAction { name, args, body } => {
                let args_str = args.iter()
                    .map(|a| format!("String {}", a))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.push_line(&format!("String {}({}) {{", name, args_str));
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                
                // Add a default return if needed, but not required for MVP
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::Enforce { name, condition: _, crash_msg } => {
                self.push_line(&format!("if ({name} == NULL || strlen({name}) == 0) {{"));
                self.indent_level += 1;
                self.push_line(&format!("fprintf(stderr, \"{}\\n\");", crash_msg));
                self.push_line("exit(1);");
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::Assign { name, value } => {
                let val_str = self.gen_expr(value);
                self.push_line(&format!("String {} = {};", name, val_str));
            }
            Statement::Match { name, cases } => {
                let mut first = true;
                let mut closed = false;
                for case in cases {
                    if let Expr::Identifier(ident) = &case.value {
                        if ident == "Otherwise" {
                            if !first {
                                self.indent_level -= 1;
                                self.push_line("} else {");
                                self.indent_level += 1;
                            } else {
                                self.push_line("{");
                                self.indent_level += 1;
                            }
                            for b in &case.body {
                                self.gen_statement(b);
                            }
                            self.indent_level -= 1;
                            self.push_line("}");
                            closed = true;
                            continue;
                        }
                    }

                    let val_str = self.gen_expr(&case.value);
                    let if_str = if first { "if" } else { "} else if" };
                    if !first {
                        self.indent_level -= 1;
                    }
                    self.push_line(&format!("{} (strcmp({}, {}) == 0) {{", if_str, name, val_str));
                    self.indent_level += 1;
                    for b in &case.body {
                        self.gen_statement(b);
                    }
                    first = false;
                }
                if !first && !closed {
                    self.indent_level -= 1;
                    self.push_line("}");
                }
            }
            Statement::RunBackground { action_call } => {
                let val_str = self.gen_expr(action_call);
                self.push_line(&format!("// RUN BACKGROUND: {}", val_str));
                self.push_line(&format!("{}; // simulated async", val_str));
            }
            Statement::Return { value } => {
                let val_str = self.gen_expr(value);
                self.push_line(&format!("return {};", val_str));
            }
            Statement::Print { value } => {
                let val_str = self.gen_expr(value);
                if let Expr::Number(_) = value {
                    self.push_line(&format!("printf(\"%d\\n\", {});", val_str));
                } else {
                    self.push_line(&format!("printf(\"%s\\n\", {});", val_str));
                }
            }
            Statement::Give { src, dst } => {
                self.push_line(&format!("// DESTROY {} -> {}", src, dst));
                self.push_line(&format!("String {} = {};", dst, src));
                self.push_line(&format!("{} = NULL;", src));
            }
            _ => {
                self.push_line("// Unimplemented statement transpilation");
            }
        }
    }

    fn gen_expr(&self, expr: &Expr) -> String {
        match expr {
            Expr::StringLit(s) => format!("\"{}\"", s),
            Expr::Number(n) => format!("{}", n),
            Expr::Identifier(i) => i.clone(),
            Expr::Call { action, args } => {
                let args_str = args.iter()
                    .map(|a| self.gen_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                // Transform action like `auth.get_role` into `auth_get_role`
                let c_action = action.replace(".", "_");
                format!("{}({})", c_action, args_str)
            }
        }
    }
}
