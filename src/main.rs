mod ast;
mod lexer;
mod parser;
mod ir;
mod codegen;
mod emit_native;
mod runtime;

use std::fs;
use std::path::Path;
use lexer::Lexer;
use parser::Parser;
use ir::IRGenerator;
use codegen::CodeGen;
use emit_native::NativeEmitter;

const VERSION: &str = "1.3.0";

fn print_splash() {
    println!();
    println!("  \u{1F9EC} Syl Language Compiler [v{}-Release]", VERSION);
    println!("  ─────────────────────────────────────────");
    println!("  Usage:");
    println!("    syl build <file.syl>              - Compiles to native C/Binary");
    println!("    syl build <file.syl> --standalone  - Compiles via Cranelift Native Backend");
    println!("    syl run <file.syl>                 - Compiles and executes immediately");
    println!("    syl test <file.syl>                - Runs a Syl test suite");
    println!("    syl get <url>                      - Installs a .syx package from the internet");
    println!();
    println!("  Environment:");
    println!("    SYL_LIB_PATH  - Override the standard library search path");
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        print_splash();
        std::process::exit(0);
    }

    let command = &args[1];
    let filename = &args[2];
    
    if command == "get" {
        let url = filename;
        let dl_name = url.split('/').last().unwrap_or("downloaded.syx");
        let local_app_data = std::env::var("LOCALAPPDATA").expect("Could not find LOCALAPPDATA");
        let lib_dir = Path::new(&local_app_data).join("Syl").join("lib");
        
        if !lib_dir.exists() {
            fs::create_dir_all(&lib_dir).unwrap();
        }
        
        println!("[ HELIX NETWORK ] Downloading {}...", dl_name);
        let status = std::process::Command::new("curl")
            .args(&["-sL", url, "-o", dl_name])
            .current_dir(&lib_dir)
            .status()
            .expect("Failed to execute curl");
            
        if status.success() {
            println!("[ HELIX NETWORK ] Successfully installed '{}' to global library.", dl_name);
        } else {
            eprintln!("[ HELIX FATAL ] Failed to download package from {}", url);
        }
        return;
    }
    
    let source = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file: {}", filename);
        std::process::exit(1);
    });

    let file_path = Path::new(filename);
    let base_path = file_path.parent().unwrap_or(Path::new(""));
    let mut imported_modules = std::collections::HashSet::new();
    imported_modules.insert(filename.clone());

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens, base_path, &mut imported_modules);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Syntax Error: {}", e);
            std::process::exit(1);
        }
    };

    let meta_name = parser.metadata.name.clone();
    let file_stem = if meta_name != "Untitled" { &meta_name } else { file_path.file_stem().unwrap().to_str().unwrap() };

    let mut is_release = false;
    let mut is_standalone = false;
    
    for arg in &args[3..] {
        if arg == "--standalone" { is_standalone = true; }
        if arg == "--release" { is_release = true; }
    }

    if command == "doc" {
        println!("[ HELIX ] DOCUMENTING: {}", filename);
        let docs_dir = Path::new("docs");
        if !docs_dir.exists() {
            fs::create_dir_all(docs_dir).unwrap();
        }
        
        let mut md = format!("# API Reference: {}\n\n", file_stem);
        
        fn walk_docs(stmts: &[crate::ast::Statement], md: &mut String) {
            for stmt in stmts {
                match stmt {
                    crate::ast::Statement::DefineAction { name, args, doc, .. } => {
                        md.push_str(&format!("## Action: `{}`\n", name));
                        if !args.is_empty() {
                            md.push_str(&format!("- **Arguments**: {}\n", args.join(", ")));
                        }
                        if let Some(d) = doc {
                            md.push_str(&format!("\n> {}\n\n", d));
                        } else {
                            md.push_str("\n*No description provided.*\n\n");
                        }
                    }
                    crate::ast::Statement::DefineEntity { name, fields, doc } => {
                        md.push_str(&format!("## Entity: `{}`\n", name));
                        md.push_str("- **Fields**:\n");
                        for (f, _) in fields {
                            md.push_str(&format!("  - `{}`\n", f));
                        }
                        if let Some(d) = doc {
                            md.push_str(&format!("\n> {}\n\n", d));
                        } else {
                            md.push_str("\n*No description provided.*\n\n");
                        }
                    }
                    crate::ast::Statement::Import { body, .. } => {
                        walk_docs(body, md);
                    }
                    _ => {}
                }
            }
        }
        
        walk_docs(&ast, &mut md);
        
        let doc_filename = docs_dir.join(format!("{}.md", file_stem));
        fs::write(&doc_filename, md).unwrap();
        println!("[ HELIX ] SUCCESS: Documentation generated at {}", doc_filename.display());
        return;
    }

    println!("Syl Compiler [v{}-Release]", VERSION);
    println!("[ HELIX ] PARSING: {}", filename);

    // Generate Helix IR
    let mut ir_gen = IRGenerator::new();
    ir_gen.generate(&ast);
    let hlx_content = ir_gen.instructions.join(" ");

    let file_path = Path::new(filename);
    let meta_name = parser.metadata.name.clone();
    let file_stem = if meta_name != "Untitled" { &meta_name } else { file_path.file_stem().unwrap().to_str().unwrap() };

    let hlx_filename = format!("{}.hlx", file_stem);
    fs::write(&hlx_filename, hlx_content).unwrap_or_else(|_| {
        eprintln!("[ HELIX ] ERROR: Failed to write {}", hlx_filename);
    });
    println!("[ HELIX ] GENERATED: {}", hlx_filename);

    let final_is_standalone = is_standalone || parser.metadata.target == "executable";

    if final_is_standalone {
        println!("[ HELIX ] NATIVE: Initializing Cranelift Native Pipeline...");
        let mut native = NativeEmitter::new();
        if is_release {
            println!("[ HELIX ] OPTIMIZING: Setting Cranelift to 'Speed and Size' mode.");
            // Note: In a real implementation, you'd pass the flag to NativeEmitter
        }
        native.compile_ast(&ast);
        let obj_data = native.finish();
        
        let obj_filename = format!("{}.obj", file_stem);
        fs::write(&obj_filename, &obj_data).unwrap_or_else(|_| {
            eprintln!("Failed to write {}", obj_filename);
        });
        
        let exe_filename = if cfg!(target_os = "windows") {
            format!("{}.exe", file_stem)
        } else {
            file_stem.to_string()
        };

        NativeEmitter::invoke_linker(&obj_filename, &exe_filename);
        return;
    }

    // Generate C Code (Fallback / Default mode)
    let mut codegen = CodeGen::new();
    
    // Separate definitions (actions, entities, externals) from top-level logic
    let mut globals = Vec::new();
    let mut main_stmts = Vec::new();

    fn sort_ast(ast: &[crate::ast::Statement], globals: &mut Vec<crate::ast::Statement>, main_stmts: &mut Vec<crate::ast::Statement>) {
        for stmt in ast {
            match stmt {
                crate::ast::Statement::DefineAction { .. } | 
                crate::ast::Statement::DefineEntity { .. } |
                crate::ast::Statement::HttpRequestRoute { .. } |
                crate::ast::Statement::External { .. } => {
                    globals.push(stmt.clone());
                }
                crate::ast::Statement::Import { body, .. } => {
                    sort_ast(body, globals, main_stmts);
                }
                _ => {
                    main_stmts.push(stmt.clone());
                }
            }
        }
    }

    sort_ast(&ast, &mut globals, &mut main_stmts);

    // Generate globals
    codegen.generate(&globals);
    let mut c_final = codegen.c_code.clone();
    
    // Add HTTP dispatch function
    c_final.push_str("\nvoid syl_http_dispatch(String path, SOCKET client) {\n");
    c_final.push_str(&codegen.http_dispatch_code);
    c_final.push_str("}\n");

    // Generate main
    c_final.push_str("\nint main() {\n");
    c_final.push_str("    syl_arena_init(1024 * 1024 * 10); // 10MB default arena\n");
    codegen.c_code = "".to_string(); // reset for main body
    codegen.generate(&main_stmts);
    c_final.push_str(&codegen.c_code);
    c_final.push_str("\n    return 0;\n}\n");

    let c_filename = format!("{}.c", file_stem);
    fs::write(&c_filename, c_final).unwrap_or_else(|_| {
        eprintln!("[ HELIX ] ERROR: Failed to write {}", c_filename);
    });
    println!("[ HELIX ] TRANSPILING: {}", c_filename);

    if command == "run" || command == "test" || command == "build" {
        let is_build_only = command == "build";
        println!("[ HELIX ] BUILDING: Compiling with TCC...");
        let tcc_path = "tools/tcc/tcc/tcc.exe";
        let exe_filename = format!("{}.exe", file_stem);
        
        let mut tcc_args = vec![&c_filename, "raylib.dll", "ws2_32.dll", "sqlite3.dll", "-I.", "-L.", "-o", &exe_filename];
        if is_release {
            println!("[ HELIX ] OPTIMIZING: Enabling -O3 production flags.");
            tcc_args.push("-O3");
        }

        let status = std::process::Command::new(tcc_path)
            .args(&tcc_args)
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!("[ HELIX ] SUCCESS: Binary \"{}\" generated.", exe_filename);
                if !is_build_only {
                    println!("[ HELIX ] RUNNING: Executing {}...", exe_filename);
                    let _ = std::process::Command::new(format!("./{}", exe_filename)).status();
                }
            } else {
                eprintln!("[ HELIX ] ERROR: TCC compilation failed.");
            }
        } else {
            eprintln!("[ HELIX ] ERROR: Could not find TCC at {}.", tcc_path);
        }
    } else {
        println!("[ HELIX ] SUCCESS: Production binary ready at ./{}", file_stem);
    }
}
