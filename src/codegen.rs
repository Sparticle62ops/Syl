use crate::ast::*;

pub struct CodeGen {
    pub c_code: String,
    pub http_dispatch_code: String,
    indent_level: usize,
    pub var_types: std::collections::HashMap<String, String>,
    pub use_sqlite: bool,
    pub use_net: bool,
}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {
            c_code: "".to_string(),
            http_dispatch_code: "".to_string(),
            indent_level: 0,
            var_types: std::collections::HashMap::new(),
            use_sqlite: false,
            use_net: false,
        }
    }

    fn get_preamble(&self, use_sqlite: bool, use_net: bool) -> String {
        let mut sqlite_include = "";
        let mut sqlite_typedef = "typedef void* Database;\n";
        let mut sqlite_funcs = r#"
Database syl_db_connect(String path) {
    return NULL;
}
void syl_db_execute(Database db, String query) {
}
"#;

        if use_sqlite {
            sqlite_include = "#include <sqlite3.h>\n";
            sqlite_typedef = "typedef sqlite3* Database;\n";
            sqlite_funcs = r#"
Database syl_db_connect(String path) {
    sqlite3* db;
    int rc = sqlite3_open(path, &db);
    if (rc) {
        _syl_last_error = "Failed to open database";
        return NULL;
    }
    _syl_last_error = NULL;
    return db;
}

void syl_db_execute(Database db, String query) {
    char* err_msg = 0;
    int rc = sqlite3_exec(db, query, 0, 0, &err_msg);
    if (rc != SQLITE_OK) {
        _syl_last_error = syl_strdup(err_msg);
        sqlite3_free(err_msg);
    } else {
        _syl_last_error = NULL;
    }
}
"#;
        }

        let mut net_include = "";
        let mut net_typedefs = "typedef int SOCKET;\n#define INVALID_SOCKET -1\n#define closesocket close\n";
        let mut net_funcs = r#"
void syl_http_reply(SOCKET client, String content) {}
SOCKET _syl_current_client = 0;
String _syl_current_path = "";
"#;

        if use_net {
            net_include = r#"
#ifdef _WIN32
    #include <winsock2.h>
    #pragma comment(lib, "ws2_32.lib")
#else
    #include <sys/socket.h>
    #include <netinet/in.h>
    #include <arpa/inet.h>
    #include <unistd.h>
#endif
"#;
            net_typedefs = r#"
#ifdef _WIN32
    // SOCKET is defined in winsock2.h
#else
    typedef int SOCKET;
    #define INVALID_SOCKET -1
    #define closesocket close
#endif
"#;
            net_funcs = r#"
void syl_http_reply(SOCKET client, String content) {
    char header[512];
    sprintf(header, "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: %d\r\n\r\n", (int)strlen(content));
    send(client, header, strlen(header), 0);
    send(client, content, strlen(content), 0);
}
SOCKET _syl_current_client;
String _syl_current_path;
"#;
        }

        let preamble = format!(r#"#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <dirent.h>
#include <sys/stat.h>
#include <math.h>
#include <time.h>
{}
#ifdef _WIN32
    #define NOGDI
    #define NOUSER
    #include <windows.h>
    #include <conio.h>
#else
    #include <termios.h>
#endif
{}
#include "raylib.h"

// Syl v0.1 C Transpiler Preamble
typedef char* String;
String _syl_last_error = NULL;
void* syl_alloc(size_t size);
char* syl_strdup(const char* s);

{}
{}
{}
{}
double syl_time_now() {{
    return (double)time(NULL);
}}

String syl_date_now() {{
    time_t t = time(NULL);
    struct tm *tm = localtime(&t);
    char* s = (char*)syl_alloc(64);
    strftime(s, 64, "%Y-%m-%d", tm);
    return s;
}}


typedef struct {{
    uint8_t* buffer;
    size_t offset;
    size_t capacity;
}} SylArena;

SylArena global_arena;

void syl_arena_init(size_t capacity) {{
    global_arena.buffer = (uint8_t*)malloc(capacity);
    global_arena.offset = 0;
    global_arena.capacity = capacity;
}}

void* syl_alloc(size_t size) {{
    if (global_arena.offset + size > global_arena.capacity) {{
        fprintf(stderr, "[ HELIX FATAL ] Arena Out of Memory.\n");
        exit(1);
    }}
    void* ptr = global_arena.buffer + global_arena.offset;
    global_arena.offset += size;
    return ptr;
}}

char* syl_strdup(const char* s) {{
    if (!s) return "";
    size_t len = strlen(s);
    char* d = (char*)syl_alloc(len + 1);
    if (d) {{
        memcpy(d, s, len + 1);
    }}
    return d;
}}

int ends_with(char* str, char* suffix) {{
    if (!str || !suffix) return 0;
    size_t lenstr = strlen(str);
    size_t lensuffix = strlen(suffix);
    if (lensuffix > lenstr) return 0;
    return strncmp(str + lenstr - lensuffix, suffix, lensuffix) == 0;
}}

typedef struct {{
    String* keys;
    String* values;
    int count;
    int capacity;
}} Dictionary;

Dictionary syl_dict_create() {{
    Dictionary d;
    d.capacity = 128;
    d.count = 0;
    d.keys = (String*)syl_alloc(sizeof(String) * d.capacity);
    d.values = (String*)syl_alloc(sizeof(String) * d.capacity);
    return d;
}}

void syl_dict_set(Dictionary* d, String key, String value) {{
    for (int i = 0; i < d->count; i++) {{
        if (strcmp(d->keys[i], key) == 0) {{
            d->values[i] = value;
            return;
        }}
    }}
    if (d->count >= d->capacity) return;
    d->keys[d->count] = key;
    d->values[d->count] = value;
    d->count++;
}}

String syl_dict_get(Dictionary d, String key) {{
    for (int i = 0; i < d.count; i++) {{
        if (strcmp(d.keys[i], key) == 0) {{
            return d.values[i];
        }}
    }}
    return "";
}}

String syl_json_from_dict(Dictionary d) {{
    char* buffer = (char*)syl_alloc(8192);
    strcpy(buffer, "{{");
    for (int i = 0; i < d.count; i++) {{
        strcat(buffer, "\"");
        strcat(buffer, d.keys[i]);
        strcat(buffer, "\": \"");
        strcat(buffer, d.values[i]);
        strcat(buffer, "\"");
        if (i < d.count - 1) strcat(buffer, ", ");
    }}
    strcat(buffer, "}}");
    return buffer;
}}

Dictionary syl_dict_from_json(String j) {{
    Dictionary d = syl_dict_create();
    char* copy = strdup(j); // Use system strdup for scratch
    char* p = copy;
    while (*p) {{
        if (*p == '"') {{
            p++;
            char* key = p;
            while (*p && *p != '"') p++;
            if (*p) {{ *p = 0; p++; }}
            while (*p && (*p == ':' || *p == ' ' || *p == '"')) p++;
            char* val = p;
            while (*p && *p != '"') p++;
            if (*p) {{ *p = 0; p++; }}
            syl_dict_set(&d, syl_strdup(key), syl_strdup(val));
        }} else p++;
    }}
    free(copy);
    return d;
}}
"#, sqlite_include, net_include, sqlite_typedef, sqlite_funcs, net_typedefs, net_funcs);
        preamble
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn push_line(&mut self, line: &str) {
        let ind = self.indent();
        self.c_code.push_str(&format!("{}{}\n", ind, line));
    }

    pub fn generate_preamble(&mut self, ast: &[Statement]) {
        let use_sqlite = has_db_usage(ast);
        let use_net = has_net_usage(ast);
        self.c_code = self.get_preamble(use_sqlite, use_net);
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
            Statement::DefineAction { name, args, body, .. } => {
                let args_str = args.iter()
                    .map(|a| {
                        self.var_types.insert(a.clone(), "double".to_string());
                        format!("double {}", a)
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                self.push_line(&format!("double {}({}) {{", name, args_str));
                self.indent_level += 1;
                self.push_line("size_t _arena_save = global_arena.offset;");
                for b in body {
                    self.gen_statement(b);
                }
                
                // Add a default return if needed, but not required for MVP
                self.push_line("global_arena.offset = _arena_save;");
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
                let is_new = !self.var_types.contains_key(name);
                
                let v_type = if let Expr::Number(_) = value {
                    "double".to_string()
                } else if let Expr::StringLit(_) = value {
                    "String".to_string()
                } else if let Expr::Join { .. } = value {
                    "String".to_string()
                } else {
                    self.var_types.get(name).cloned().unwrap_or("double".to_string())
                };

                if is_new {
                    self.var_types.insert(name.clone(), v_type.clone());
                    self.push_line(&format!("{} {} = {};", v_type, name, val_str));
                } else {
                    self.push_line(&format!("{} = {};", name, val_str));
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
                let v_type = if let Expr::Number(_) = value {
                    "double".to_string()
                } else if let Expr::StringLit(_) = value {
                    "String".to_string()
                } else if let Expr::Identifier(id) = value {
                    self.var_types.get(id).cloned().unwrap_or("double".to_string())
                } else {
                    "double".to_string()
                };

                if v_type == "double" {
                    self.push_line(&format!("printf(\"%g\\n\", (double)({}));", val_str));
                } else {
                    self.push_line(&format!("printf(\"%s\\n\", {});", val_str));
                }
            }
            Statement::Verify { left, right } => {
                let left_str = self.gen_expr(left);
                let right_str = self.gen_expr(right);
                
                let v_type = if let Expr::Number(_) = left {
                    "double".to_string()
                } else if let Expr::StringLit(_) = left {
                    "String".to_string()
                } else if let Expr::StringLit(_) = right {
                    "String".to_string()
                } else if let Expr::Identifier(id) = left {
                    self.var_types.get(id).cloned().unwrap_or("double".to_string())
                } else {
                    "double".to_string()
                };

                if v_type == "String" {
                    self.push_line(&format!("if (strcmp({}, {}) != 0) {{", left_str, right_str));
                    self.push_line(&format!("    printf(\"[ TEST FAILED ] Expected %s, got %s\\n\", {}, {});", right_str, left_str));
                    self.push_line("    exit(1);");
                    self.push_line("} else {");
                    self.push_line("    printf(\"✔ Test passed.\\n\");");
                    self.push_line("}");
                } else {
                    self.push_line(&format!("if ({} != {}) {{", left_str, right_str));
                    self.push_line(&format!("    printf(\"[ TEST FAILED ] Expected %g, got %g\\n\", (double)({}), (double)({}));", right_str, left_str));
                    self.push_line("    exit(1);");
                    self.push_line("} else {");
                    self.push_line("    printf(\"✔ Test passed.\\n\");");
                    self.push_line("}");
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
                self.push_line("_syl_last_error = NULL;");
                self.push_line("fseek(f, 0, SEEK_END); long fsize = ftell(f); fseek(f, 0, SEEK_SET);");
                self.push_line(&format!("{} = syl_alloc(fsize + 1);", identifier));
                self.push_line(&format!("fread({}, 1, fsize, f); {}[fsize] = 0;", identifier, identifier));
                self.push_line("fclose(f);");
                self.indent_level -= 1;
                self.push_line("} else { _syl_last_error = \"File not found\"; } }");
            }
            Statement::List { path, identifier } => {
                let path_str = self.gen_expr(path);
                self.push_line(&format!("String* {} = syl_alloc(1024 * sizeof(String));", identifier));
                self.push_line(&format!("int {}_count = 0;", identifier));
                self.push_line(&format!("{{ DIR *d = opendir({}); if (d) {{", path_str));
                self.indent_level += 1;
                self.push_line("_syl_last_error = NULL;");
                self.push_line("struct dirent *dir;");
                self.push_line(&format!("while ((dir = readdir(d)) != NULL) {{ {}[{}_count++] = syl_strdup(dir->d_name); }}", identifier, identifier));
                self.push_line("closedir(d);");
                self.indent_level -= 1;
                self.push_line("} else { _syl_last_error = \"Directory not found\"; } }");
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
                self.push_line(&format!("String* {} = syl_alloc(1024 * sizeof(String));", name));
                self.push_line(&format!("int {}_count = 0;", name));
                for item in items {
                    let val = self.gen_expr(item);
                    self.push_line(&format!("{}[{}_count++] = syl_strdup({});", name, name, val));
                }
            }
            Statement::AddToList { item, list_name } => {
                let val = self.gen_expr(item);
                self.push_line(&format!("{}[{}_count++] = syl_strdup({});", list_name, list_name, val));
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
                    let c_color = color.trim_matches('"').to_uppercase();
                    self.push_line(&format!("ClearBackground({});", c_color));
                } else if c_name == "DrawCircle" {
                    let x = self.gen_expr(&args[0]);
                    let y = self.gen_expr(&args[1]);
                    let radius = self.gen_expr(&args[2]);
                    let color = self.gen_expr(&args[3]);
                    let c_color = color.trim_matches('"').to_uppercase();
                    self.push_line(&format!("DrawCircle({}, {}, {}, {});", x, y, radius, c_color));
                } else if c_name == "DrawRectangle" {
                    let x = self.gen_expr(&args[0]);
                    let y = self.gen_expr(&args[1]);
                    let w = self.gen_expr(&args[2]);
                    let h = self.gen_expr(&args[3]);
                    let color = self.gen_expr(&args[4]);
                    let c_color = color.trim_matches('"').to_uppercase();
                    self.push_line(&format!("DrawRectangle({}, {}, {}, {}, {});", x, y, w, h, c_color));
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
            Statement::While { condition, body } => {
                let cond_str = self.gen_expr(condition);
                self.push_line(&format!("while ({}) {{", cond_str));
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
            Statement::DefineEntity { name, fields, .. } => {
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
            Statement::ListWords { source, identifier } => {
                let src_str = self.gen_expr(source);
                self.push_line(&format!("String* {} = syl_alloc(1024 * sizeof(String));", identifier));
                self.push_line(&format!("int {}_count = 0;", identifier));
                self.push_line(&format!("{{ char* s = syl_strdup({}); char* tok = strtok(s, \" \t\\n\");", src_str));
                self.push_line(&format!("  while(tok) {{ {}[{}_count++] = syl_strdup(tok); tok = strtok(NULL, \" \t\\n\"); }} }}", identifier, identifier));
            }
            Statement::Increase { name, amount } => {
                let amt_str = self.gen_expr(amount);
                self.push_line(&format!("{} += {};", name, amt_str));
            }
            Statement::GetItem { identifier, index, list } => {
                let idx_str = self.gen_expr(index);
                self.push_line(&format!("if ({} < 1 || {} > {}_count) {{ fprintf(stderr, \"[ HELIX FATAL ] Index out of bounds: %d\\n\", {}); exit(1); }}", idx_str, idx_str, list, idx_str));
                self.push_line(&format!("String {} = {}[{}-1];", identifier, list, idx_str));
            }
            Statement::SetItem { index, list, value } => {
                let idx_str = self.gen_expr(index);
                let val_str = self.gen_expr(value);
                self.push_line(&format!("if ({} < 1 || {} > {}_count) {{ fprintf(stderr, \"[ HELIX FATAL ] Index out of bounds: %d\\n\", {}); exit(1); }}", idx_str, idx_str, list, idx_str));
                self.push_line(&format!("{}[{}-1] = syl_strdup({});", list, idx_str, val_str));
            }
            Statement::DrawRect { x, y, w, h, color } => {
                let xs = self.gen_expr(x); let ys = self.gen_expr(y);
                let ws = self.gen_expr(w); let hs = self.gen_expr(h);
                let cs = self.gen_expr(color).to_uppercase();
                self.push_line(&format!("DrawRectangle({}, {}, {}, {}, {});", xs, ys, ws, hs, cs));
            }
            Statement::DrawCircle { x, y, radius, color } => {
                let xs = self.gen_expr(x); let ys = self.gen_expr(y);
                let rs = self.gen_expr(radius);
                let cs = self.gen_expr(color).to_uppercase();
                self.push_line(&format!("DrawCircle({}, {}, {}, {});", xs, ys, rs, cs));
            }
            Statement::IfExpr { condition, then_branch, else_branch } => {
                let cond = self.gen_expr(condition);
                self.push_line(&format!("if ({}) {{", cond));
                self.indent_level += 1;
                for b in then_branch {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                if let Some(elb) = else_branch {
                    self.push_line("} else {");
                    self.indent_level += 1;
                    for b in elb {
                        self.gen_statement(b);
                    }
                    self.indent_level -= 1;
                }
                self.push_line("}");
            }
            Statement::CreateDictionary { name } => {
                self.var_types.insert(name.clone(), "Dictionary".to_string());
                self.push_line(&format!("Dictionary {} = syl_dict_create();", name));
            }
            Statement::SetDictKey { dict, key, value } => {
                let k = self.gen_expr(key);
                let v = self.gen_expr(value);
                self.push_line(&format!("syl_dict_set(&{}, {}, {});", dict, k, v));
            }
            Statement::ListenHttp { port } => {
                let p = self.gen_expr(port);
                self.push_line(&format!("// SYL HTTP SERVER BOILERPLATE"));
                self.push_line("{");
                self.indent_level += 1;
                self.push_line("WSADATA wsa; WSAStartup(MAKEWORD(2,2), &wsa);");
                self.push_line(&format!("SOCKET server = socket(AF_INET, SOCK_STREAM, 0);"));
                self.push_line("struct sockaddr_in saddr; saddr.sin_family = AF_INET; saddr.sin_addr.s_addr = INADDR_ANY;");
                self.push_line(&format!("saddr.sin_port = htons({});", p));
                self.push_line("bind(server, (struct sockaddr *)&saddr, sizeof(saddr));");
                self.push_line("listen(server, 3);");
                self.push_line("while(1) {");
                self.indent_level += 1;
                self.push_line("struct sockaddr_in client; int csize = sizeof(client);");
                self.push_line("_syl_current_client = accept(server, (struct sockaddr *)&client, &csize);");
                self.push_line("if (_syl_current_client != INVALID_SOCKET) {");
                self.indent_level += 1;
                self.push_line("char buffer[4096] = {0}; recv(_syl_current_client, buffer, 4096, 0);");
                self.push_line("char m[16], p[256], pr[16]; sscanf(buffer, \"%s %s %s\", m, p, pr);");
                self.push_line("_syl_current_path = syl_strdup(p);");
                self.push_line("syl_http_dispatch(_syl_current_path, _syl_current_client);");
                self.push_line("closesocket(_syl_current_client);");
                self.indent_level -= 1;
                self.push_line("}");
                self.indent_level -= 1;
                self.push_line("}");
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::HttpRequestRoute { path, body } => {
                let p = self.gen_expr(path);
                // We generate this into http_dispatch_code instead of c_code
                let mut temp_code = String::new();
                std::mem::swap(&mut self.c_code, &mut temp_code);
                
                self.push_line(&format!("if (strcmp(path, {}) == 0) {{", p));
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
                
                std::mem::swap(&mut self.c_code, &mut self.http_dispatch_code);
                self.http_dispatch_code.push_str(&temp_code);
                std::mem::swap(&mut self.c_code, &mut temp_code); // restore c_code
            }
            Statement::HttpReply { content } => {
                let c = self.gen_expr(content);
                self.push_line(&format!("syl_http_reply(_syl_current_client, {});", c));
            }
            Statement::Attempt { action } => {
                self.push_line("_syl_last_error = NULL;");
                self.gen_statement(action);
            }
            Statement::IfFailed { body } => {
                self.push_line("if (_syl_last_error != NULL) {");
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::IfSucceeded { body } => {
                self.push_line("if (_syl_last_error == NULL) {");
                self.indent_level += 1;
                for b in body {
                    self.gen_statement(b);
                }
                self.indent_level -= 1;
                self.push_line("}");
            }
            Statement::ConnectDB { path, identifier } => {
                let p = self.gen_expr(path);
                self.var_types.insert(identifier.clone(), "Database".to_string());
                self.push_line(&format!("Database {} = syl_db_connect({});", identifier, p));
            }
            Statement::ExecuteQuery { query, db_identifier, results_list } => {
                let q = self.gen_expr(query);
                if let Some(_list) = results_list {
                    self.push_line(&format!("syl_db_execute({}, {});", db_identifier, q));
                } else {
                    self.push_line(&format!("syl_db_execute({}, {});", db_identifier, q));
                }
            }
            Statement::ClearTerminal => {
                self.push_line("printf(\"\\x1B[2J\\x1B[1;1H\");");
            }
            Statement::WaitForKeyPress { var } => {
                self.var_types.insert(var.clone(), "String".to_string());
                self.push_line(&format!("String {} = \"\";", var));
                self.push_line(&format!("#ifdef _WIN32"));
                self.push_line(&format!("if (_kbhit()) {{"));
                self.push_line(&format!("    int c = _getch();"));
                self.push_line(&format!("    if (c == 224 || c == 0) {{"));
                self.push_line(&format!("        c = _getch();"));
                self.push_line(&format!("        if (c == 72) {} = \"w\";", var));
                self.push_line(&format!("        else if (c == 80) {} = \"s\";", var));
                self.push_line(&format!("        else if (c == 75) {} = \"a\";", var));
                self.push_line(&format!("        else if (c == 77) {} = \"d\";", var));
                self.push_line(&format!("    }} else {{"));
                self.push_line(&format!("        char buf[2] = {{c, 0}};"));
                self.push_line(&format!("        {} = syl_strdup(buf);", var));
                self.push_line(&format!("    }}"));
                self.push_line(&format!("}}"));
                self.push_line(&format!("#else"));
                self.push_line(&format!("{{"));
                self.push_line(&format!("    struct termios oldt, newt;"));
                self.push_line(&format!("    tcgetattr(STDIN_FILENO, &oldt);"));
                self.push_line(&format!("    newt = oldt;"));
                self.push_line(&format!("    newt.c_lflag &= ~(ICANON | ECHO);"));
                self.push_line(&format!("    tcsetattr(STDIN_FILENO, TCSANOW, &newt);"));
                self.push_line(&format!("    struct timeval tv = {{0, 0}};"));
                self.push_line(&format!("    fd_set fds;"));
                self.push_line(&format!("    FD_ZERO(&fds);"));
                self.push_line(&format!("    FD_SET(STDIN_FILENO, &fds);"));
                self.push_line(&format!("    int has_data = select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv);"));
                self.push_line(&format!("    if (has_data > 0) {{"));
                self.push_line(&format!("        int c = getchar();"));
                self.push_line(&format!("        if (c == 27) {{"));
                self.push_line(&format!("            FD_ZERO(&fds);"));
                self.push_line(&format!("            FD_SET(STDIN_FILENO, &fds);"));
                self.push_line(&format!("            if (select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv) > 0) {{"));
                self.push_line(&format!("                int c2 = getchar();"));
                self.push_line(&format!("                if (c2 == '[') {{"));
                self.push_line(&format!("                    FD_ZERO(&fds);"));
                self.push_line(&format!("                    FD_SET(STDIN_FILENO, &fds);"));
                self.push_line(&format!("                    if (select(STDIN_FILENO + 1, &fds, NULL, NULL, &tv) > 0) {{"));
                self.push_line(&format!("                        int c3 = getchar();"));
                self.push_line(&format!("                        if (c3 == 'A') {} = \"w\";", var));
                self.push_line(&format!("                        else if (c3 == 'B') {} = \"s\";", var));
                self.push_line(&format!("                        else if (c3 == 'D') {} = \"a\";", var));
                self.push_line(&format!("                        else if (c3 == 'C') {} = \"d\";", var));
                self.push_line(&format!("                    }}"));
                self.push_line(&format!("                }}"));
                self.push_line(&format!("            }}"));
                self.push_line(&format!("        }} else {{"));
                self.push_line(&format!("            char buf[2] = {{c, 0}};"));
                self.push_line(&format!("            {} = syl_strdup(buf);", var));
                self.push_line(&format!("        }}"));
                self.push_line(&format!("    }}"));
                self.push_line(&format!("    tcsetattr(STDIN_FILENO, TCSANOW, &oldt);"));
                self.push_line(&format!("}}"));
                self.push_line(&format!("#endif"));
            }
            Statement::SleepMilliseconds { duration } => {
                let dur = self.gen_expr(duration);
                self.push_line(&format!("#ifdef _WIN32"));
                self.push_line(&format!("Sleep((DWORD)({}));", dur));
                self.push_line(&format!("#else"));
                self.push_line(&format!("usleep((useconds_t)(({}) * 1000));", dur));
                self.push_line(&format!("#endif"));
            }
            Statement::PrintColored { text, color } => {
                let text_val = self.gen_expr(text);
                let color_code = match color.as_str() {
                    "red" => "\\x1b[31m",
                    "green" => "\\x1b[32m",
                    "yellow" => "\\x1b[33m",
                    "blue" => "\\x1b[34m",
                    "magenta" => "\\x1b[35m",
                    "cyan" => "\\x1b[36m",
                    "white" => "\\x1b[37m",
                    _ => "\\x1b[0m",
                };
                self.push_line(&format!("printf(\"{color_code}%s\\x1b[0m\\n\", {text_val});"));
            }
            Statement::MoveCursor { x, y } => {
                let x_val = self.gen_expr(x);
                let y_val = self.gen_expr(y);
                // ANSI cursor positioning is 1-indexed (Row;Column) so (Y;X)
                self.push_line(&format!("printf(\"\\x1B[%d;%dH\", (int)({}) + 1, (int)({}) + 1);", y_val, x_val));
            }
            Statement::DrawTerminalBox { x, y, w, h } => {
                let x_val = self.gen_expr(x);
                let y_val = self.gen_expr(y);
                let w_val = self.gen_expr(w);
                let h_val = self.gen_expr(h);
                self.push_line(&format!("{{ int _bx = {}; int _by = {}; int _bw = {}; int _bh = {};", x_val, y_val, w_val, h_val));
                // Top border
                self.push_line("printf(\"\\x1B[%d;%dH┌\", _by + 1, _bx + 1);");
                self.push_line("for(int i=1; i<_bw-1; i++) printf(\"─\");");
                self.push_line("printf(\"┐\\n\");");
                // Sides
                self.push_line("for(int i=1; i<_bh-1; i++) {");
                self.push_line("  printf(\"\\x1B[%d;%dH│\", _by + 1 + i, _bx + 1);");
                self.push_line("  printf(\"\\x1B[%d;%dH│\", _by + 1 + i, _bx + _bw);");
                self.push_line("}");
                // Bottom border
                self.push_line("printf(\"\\x1B[%d;%dH└\", _by + _bh, _bx + 1);");
                self.push_line("for(int i=1; i<_bw-1; i++) printf(\"─\");");
                self.push_line("printf(\"┘\\n\"); }");
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
                let c_action = action.replace(".", "_");
                format!("{}({})", c_action, args_str)
            }
            Expr::GetField { field_name, entity_instance } => {
                format!("{}.{}", entity_instance, field_name)
            }
            Expr::Join { left, right } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                format!("({{ char* res = syl_alloc(strlen({}) + strlen({}) + 1); strcpy(res, {}); strcat(res, {}); res; }})", l, r, l, r)
            }
            Expr::Sqrt { value } => {
                format!("sqrt({})", self.gen_expr(value))
            }
            Expr::Pow { base, exponent } => {
                format!("pow({}, {})", self.gen_expr(base), self.gen_expr(exponent))
            }
            Expr::Random { min, max } => {
                format!("(rand() % ({} - {} + 1) + {})", self.gen_expr(max), self.gen_expr(min), self.gen_expr(min))
            }
            Expr::FileSize { path } => {
                format!("({{ struct stat st; stat({}, &st); (int)st.st_size; }})", self.gen_expr(path))
            }
            // v1.4: Math block binary operations
            Expr::BinaryOp { op, left, right } => {
                let l = self.gen_expr(left);
                let r = self.gen_expr(right);
                let op_str = match op {
                    BinOp::Add => "+",
                    BinOp::Sub => "-",
                    BinOp::Mul => "*",
                    BinOp::Div => "/",
                    BinOp::Mod => "%",
                    BinOp::Lt  => "<",
                    BinOp::Gt  => ">",
                    BinOp::Eq  => "==",
                    BinOp::Neq => "!=",
                    BinOp::Lte => "<=",
                    BinOp::Gte => ">=",
                    BinOp::And => "&&",
                    BinOp::Or  => "||",
                };
                format!("({} {} {})", l, op_str, r)
            }
            Expr::UnaryNeg { value } => {
                format!("(-{})", self.gen_expr(value))
            }
            // v1.4: Raylib intrinsics
            Expr::DeltaTime => "GetFrameTime()".to_string(),
            Expr::MouseX => "GetMouseX()".to_string(),
            Expr::MouseY => "GetMouseY()".to_string(),
            Expr::GetDictKey { dict, key } => {
                format!("syl_dict_get({}, {})", dict, self.gen_expr(key))
            }
            Expr::JsonFromDict { dict } => {
                format!("syl_json_from_dict({})", dict)
            }
            Expr::DictFromJson { json } => {
                format!("syl_dict_from_json({})", self.gen_expr(json))
            }
            Expr::CurrentTime => "syl_time_now()".to_string(),
            Expr::CurrentDate => "syl_date_now()".to_string(),
        }
    }
}

fn has_db_usage(ast: &[Statement]) -> bool {
    for stmt in ast {
        match stmt {
            Statement::ConnectDB { .. } | Statement::ExecuteQuery { .. } => return true,
            Statement::DefineAction { body, .. } => {
                if has_db_usage(body) { return true; }
            }
            Statement::While { body, .. } | Statement::WhileNot { body, .. } => {
                if has_db_usage(body) { return true; }
            }
            Statement::Repeat { body, .. } | Statement::ForEach { body, .. } => {
                if has_db_usage(body) { return true; }
            }
            Statement::IfExpr { then_branch, else_branch, .. } => {
                if has_db_usage(then_branch) { return true; }
                if let Some(else_b) = else_branch {
                    if has_db_usage(else_b) { return true; }
                }
            }
            Statement::IfEndsWith { body, .. } | Statement::IfKeyPressed { body, .. } => {
                if has_db_usage(body) { return true; }
            }
            _ => {}
        }
    }
    false
}

fn has_net_usage(ast: &[Statement]) -> bool {
    for stmt in ast {
        match stmt {
            Statement::HttpRequestRoute { .. } => return true,
            Statement::DefineAction { body, .. } => {
                if has_net_usage(body) { return true; }
            }
            Statement::While { body, .. } | Statement::WhileNot { body, .. } => {
                if has_net_usage(body) { return true; }
            }
            Statement::Repeat { body, .. } | Statement::ForEach { body, .. } => {
                if has_net_usage(body) { return true; }
            }
            Statement::IfExpr { then_branch, else_branch, .. } => {
                if has_net_usage(then_branch) { return true; }
                if let Some(else_b) = else_branch {
                    if has_net_usage(else_b) { return true; }
                }
            }
            Statement::IfEndsWith { body, .. } | Statement::IfKeyPressed { body, .. } => {
                if has_net_usage(body) { return true; }
            }
            _ => {}
        }
    }
    false
}

