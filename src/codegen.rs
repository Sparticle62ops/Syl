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
#include <dirent.h>
#include <sys/stat.h>
#include "raylib.h"

// Syl v0.1 C Transpiler Preamble
typedef char* String;

int ends_with(char* str, char* suffix) {
    if (!str || !suffix) return 0;
    size_t lenstr = strlen(str);
    size_t lensuffix = strlen(suffix);
    if (lensuffix > lenstr) return 0;
    return strncmp(str + lenstr - lensuffix, suffix, lensuffix) == 0;
}
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
            Statement::Import { filename, alias, body } => {
                self.push_line(&format!("// Bring in {} as {}", filename, alias));
                for b in body {
                    self.gen_statement(b);
                }
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
                if let Expr::Number(_) = value {
                    self.push_line(&format!("int {} = {};", name, val_str));
                } else {
                    self.push_line(&format!("String {} = {};", name, val_str));
                }
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
            Statement::Write { data, path } => {
                let data_str = self.gen_expr(data);
                let path_str = self.gen_expr(path);
                self.push_line(&format!("{{ FILE *f = fopen({}, \"w\"); if (f) {{ fprintf(f, \"%s\", {}); fclose(f); }} }}", path_str, data_str));
            }
            Statement::Read { path, identifier } => {
                let path_str = self.gen_expr(path);
                self.push_line(&format!("String {} = \"\";", identifier));
                self.push_line(&format!("{{ FILE *f = fopen({}, \"r\"); if (f) {{", path_str));
                self.indent_level += 1;
                self.push_line("fseek(f, 0, SEEK_END); long fsize = ftell(f); fseek(f, 0, SEEK_SET);");
                self.push_line(&format!("{} = malloc(fsize + 1);", identifier));
                self.push_line(&format!("fread({}, 1, fsize, f); {}[fsize] = 0;", identifier, identifier));
                self.push_line("fclose(f);");
                self.indent_level -= 1;
                self.push_line("} }");
            }
            Statement::List { path, identifier } => {
                let path_str = self.gen_expr(path);
                self.push_line(&format!("String* {} = malloc(1024 * sizeof(String));", identifier));
                self.push_line(&format!("int {}_count = 0;", identifier));
                self.push_line(&format!("{{ DIR *d = opendir({}); if (d) {{", path_str));
                self.indent_level += 1;
                self.push_line("struct dirent *dir;");
                self.push_line(&format!("while ((dir = readdir(d)) != NULL) {{ {}[{}_count++] = strdup(dir->d_name); }}", identifier, identifier));
                self.push_line("closedir(d);");
                self.indent_level -= 1;
                self.push_line("} }");
            }
            Statement::Move { path, destination } => {
                let path_str = self.gen_expr(path);
                let dest_str = self.gen_expr(destination);
                self.push_line(&format!("rename({}, {});", path_str, dest_str));
            }
            Statement::CreateFolder { path } => {
                let path_str = self.gen_expr(path);
                self.push_line(&format!("mkdir({}, 0777);", path_str));
            }
            Statement::IfEndsWith { filename, extension, body } => {
                self.push_line(&format!("if (ends_with({}, \"{}\")) {{", filename, extension));
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::ForEach { item, collection, body } => {
                self.push_line(&format!("for (int _i = 0; _i < {}_count; _i++) {{", collection));
                self.indent_level += 1;
                self.push_line(&format!("String {} = {}[_i];", item, collection));
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::External { namespace, action, lib_path } => {
                self.push_line(&format!("// External {}_{} from {}", namespace, action, lib_path));
            }
            Statement::Execute { namespace, action, args } => {
                let args_str = args.iter()
                    .map(|a| self.gen_expr(a))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.push_line(&format!("{}_{}({});", namespace, action, args_str));
            }
            Statement::CreateList { name, items } => {
                self.push_line(&format!("String* {} = malloc(1024 * sizeof(String));", name));
                self.push_line(&format!("int {}_count = 0;", name));
                for item in items {
                    let val = self.gen_expr(item);
                    self.push_line(&format!("{}[{}_count++] = strdup({});", name, name, val));
                }
            }
            Statement::AddToList { item, list_name } => {
                let val = self.gen_expr(item);
                self.push_line(&format!("{}[{}_count++] = strdup({});", list_name, list_name, val));
            }
            Statement::CallAction { name, args } => {
                let c_name = if name.starts_with("ui_") { &name[3..] } else { name };
                if c_name == "DrawText" {
                    let x = self.gen_expr(&args[args.len()-2]);
                    let y = self.gen_expr(&args[args.len()-1]);
                    let text_parts = &args[0..args.len()-2];
                    
                    if text_parts.len() == 1 {
                        let text = self.gen_expr(&text_parts[0]);
                        self.push_line(&format!("DrawText({}, {}, {}, 20, RAYWHITE);", text, x, y));
                    } else {
                        let mut fmt = String::new();
                        let mut vals = Vec::new();
                        for part in text_parts {
                            match part {
                                Expr::StringLit(_) => {
                                    fmt.push_str("%s");
                                    vals.push(self.gen_expr(part));
                                }
                                _ => {
                                    fmt.push_str("%d");
                                    vals.push(self.gen_expr(part));
                                }
                            }
                        }
                        self.push_line(&format!("DrawText(TextFormat(\"{}\", {}), {}, {}, 20, RAYWHITE);", fmt, vals.join(", "), x, y));
                    }
                } else if c_name == "ClearBackground" {
                    let color = self.gen_expr(&args[0]);
                    let c_color = color.to_uppercase();
                    self.push_line(&format!("ClearBackground({});", c_color));
                } else {
                    let args_str = args.iter()
                        .map(|a| self.gen_expr(a))
                        .collect::<Vec<_>>()
                        .join(", ");
                    self.push_line(&format!("{}({});", c_name, args_str));
                }
            }
            Statement::WhileNot { condition_action, body } => {
                let c_cond = if condition_action.starts_with("ui_") { &condition_action[3..] } else { &condition_action };
                self.push_line(&format!("while (!{}()) {{", c_cond));
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::IfKeyPressed { key, body } => {
                let key_const = format!("KEY_{}", key.to_uppercase());
                self.push_line(&format!("if (IsKeyPressed({})) {{", key_const));
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::DefineEntity { name, fields } => {
                self.push_line(&format!("typedef struct {{"));
                self.indent_level += 1;
                for (f_name, f_val) in fields {
                    if let Expr::Number(_) = f_val {
                        self.push_line(&format!("int {};", f_name));
                    } else {
                        self.push_line(&format!("String {};", f_name));
                    }
                }
                self.indent_level -= 1;
                self.push_line(&format!("}} {};", name));
            }
            Statement::CreateEntity { entity_type, name } => {
                self.push_line(&format!("{} {} = {{0}};", entity_type, name));
            }
            Statement::SetField { field_name, entity_instance, value } => {
                let val_str = self.gen_expr(value);
                self.push_line(&format!("{}.{} = {};", entity_instance, field_name, val_str));
            }
            Statement::Download { url, target } => {
                let url_str = self.gen_expr(url);
                self.push_line(&format!("String {} = \"\"; // Download target", target));
                self.push_line(&format!("{{ char cmd[1024]; sprintf(cmd, \"curl -s %s -o tmp_download.txt\", {}); system(cmd); }}", url_str));
                self.push_line(&format!("// [ HELIX BORROWED ] {} tagged as temporary until verification.", target));
            }
            Statement::Increase { name, amount } => {
                let amt_str = self.gen_expr(amount);
                self.push_line(&format!("{} += {};", name, amt_str));
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
            Expr::GetField { field_name, entity_instance } => {
                format!("{}.{}", entity_instance, field_name)
            }
        }
    }
}
