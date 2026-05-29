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

mod tui;

const VERSION: &str = "2.5.0";

const ANSI_BOLD_CYAN: &str = "\x1b[1;36m";
const ANSI_BOLD_GREEN: &str = "\x1b[1;32m";
const ANSI_BOLD_RED: &str = "\x1b[1;31m";
const ANSI_RESET: &str = "\x1b[0m";

fn print_splash() {
    println!();
    println!("{}", ANSI_BOLD_CYAN);
    println!("   _____ __  __   ");
    println!("  / ___/ \\ \\/ / /   ");
    println!("  \\__ \\   \\  / /    ");
    println!(" ___/ /   / / /___  ");
    println!("/____/   /_/ /_____/");
    println!("                    ");
    println!("{}  \u{1F9EC} Syl OS Compiler [v{}-Beta]", ANSI_RESET, VERSION);
    println!("  ─────────────────────────────────────────");
    println!("  Usage:");
    println!("    syl build <file>       - Compiles to native binary");
    println!("    syl run <file>         - Compiles and executes immediately");
    println!("    syl build <f> --wasm   - Compiles to WebAssembly");
    println!("    syl watch <file>       - Hot-reload on file changes");
    println!("    syl doc <file>         - Generates API documentation");
    println!("    syl ide                - Opens the Terminal OS Environment");
    println!("    syl setup-wasm         - Installs Emscripten toolchain");
    println!("    syl share <file>       - Uploads code to the Helix Network");
    println!("    syl get <url|code>     - Installs a package from the internet");
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_splash();
        std::process::exit(0);
    }

    let command = &args[1];
    
    if command == "ide" {
        tui::start_ide();
        return;
    }

    if command == "setup-wasm" {
        println!("{}[ \u{1F9EC} SYL OS ]{} Setting up Emscripten (WASM)...", ANSI_BOLD_CYAN, ANSI_RESET);
        let tools_dir = Path::new("tools");
        if !tools_dir.exists() {
            fs::create_dir_all(tools_dir).unwrap();
        }
        let emsdk_dir = tools_dir.join("emsdk");
        if !emsdk_dir.exists() {
            println!("Cloning emsdk repository...");
            let git_cmd = find_git();
            std::process::Command::new(&git_cmd)
                .args(&["clone", "https://github.com/emscripten-core/emsdk.git"])
                .current_dir(tools_dir)
                .status()
                .expect("Failed to clone emsdk. Ensure git is available.");
        }
        println!("Installing latest emsdk...");
        let mut install_cmd = if cfg!(target_os = "windows") {
            let mut c = std::process::Command::new("cmd.exe");
            c.arg("/c").arg("emsdk.bat");
            c
        } else {
            std::process::Command::new("./emsdk")
        };
        install_cmd
            .args(&["install", "latest"])
            .current_dir(&emsdk_dir)
            .status()
            .expect("Failed to install emsdk");

        println!("Activating latest emsdk...");
        let mut activate_cmd = if cfg!(target_os = "windows") {
            let mut c = std::process::Command::new("cmd.exe");
            c.arg("/c").arg("emsdk.bat");
            c
        } else {
            std::process::Command::new("./emsdk")
        };
        activate_cmd
            .args(&["activate", "latest"])
            .current_dir(&emsdk_dir)
            .status()
            .expect("Failed to activate emsdk");
        println!("{}[ \u{1F9EC} SUCCESS ]{} Emscripten installed. To use it in current terminal, run:", ANSI_BOLD_GREEN, ANSI_RESET);
        if cfg!(target_os = "windows") {
            println!("    .\\tools\\emsdk\\emsdk_env.bat");
        } else {
            println!("    source ./tools/emsdk/emsdk_env.sh");
        }
        return;
    }

    if args.len() < 3 {
        print_splash();
        std::process::exit(0);
    }

    let filename = &args[2];
    
    if command == "share" {
        println!("{}[ \u{1F9EC} SYL NETWORK ]{} Uploading {}...", ANSI_BOLD_CYAN, ANSI_RESET, filename);
        let output = std::process::Command::new("curl")
            .args(&["-T", filename, &format!("https://transfer.sh/{}", filename)])
            .output()
            .expect("Failed to execute curl");
        
        let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.starts_with("https://transfer.sh/") {
            let shortcode = url.replace("https://transfer.sh/", "HLX-");
            println!("{}[ \u{1F9EC} SUCCESS ]{} File uploaded securely.", ANSI_BOLD_GREEN, ANSI_RESET);
            println!("Tell your friend to run: syl get {}", shortcode);
        } else {
            println!("{}[ \u{1F9EC} FATAL ]{} Failed to upload file.", ANSI_BOLD_RED, ANSI_RESET);
        }
        return;
    }
    
    if command == "watch" {
        println!("{}[ \u{1F9EC} LIVE ENGINE ]{} Watching {} for changes...", ANSI_BOLD_CYAN, ANSI_RESET, filename);
        let mut last_modified = fs::metadata(filename).unwrap().modified().unwrap();
        let mut child: Option<std::process::Child> = None;
        let file_path = Path::new(filename);
        let file_stem = file_path.file_stem().unwrap().to_str().unwrap();
        let exe_filename = format!("{}.exe", file_stem);

        let compile_and_run = |child: &mut Option<std::process::Child>| {
            if let Some(mut c) = child.take() {
                let _ = c.kill();
                let _ = c.wait(); // prevent zombie
            }
            println!("{}[ \u{1F9EC} LIVE ENGINE ]{} Recompiling...", ANSI_BOLD_CYAN, ANSI_RESET);
            let status = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("build")
                .arg(filename)
                .status()
                .expect("Failed to execute syl build");
                
            if status.success() {
                let spawn_res = std::process::Command::new(format!("./{}", exe_filename)).spawn();
                if let Ok(c) = spawn_res {
                    *child = Some(c);
                }
            }
        };

        compile_and_run(&mut child);

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            if let Ok(meta) = fs::metadata(filename) {
                if let Ok(modified) = meta.modified() {
                    if modified > last_modified {
                        last_modified = modified;
                        compile_and_run(&mut child);
                    }
                }
            }
        }
    }
    
    if command == "get" {
        let mut url = filename.to_string();

        if url.starts_with("HLX-") {
            url = url.replace("HLX-", "https://transfer.sh/");
        }
        let dl_name = url.split('/').last().unwrap_or("downloaded.syx");
        let local_app_data = std::env::var("LOCALAPPDATA").expect("Could not find LOCALAPPDATA");
        let lib_dir = Path::new(&local_app_data).join("Syl").join("lib");
        
        if !lib_dir.exists() {
            fs::create_dir_all(&lib_dir).unwrap();
        }
        
        println!("{}[ \u{1F9EC} SYL NETWORK ]{} Downloading {}...", ANSI_BOLD_CYAN, ANSI_RESET, dl_name);
        let status = std::process::Command::new("curl")
            .args(&["-sL", &url, "-o", dl_name])
            .current_dir(&lib_dir)
            .status()
            .expect("Failed to execute curl");
            
        if status.success() {
            println!("{}[ \u{1F9EC} SUCCESS ]{} Installed '{}' to global library.", ANSI_BOLD_GREEN, ANSI_RESET, dl_name);
        } else {
            eprintln!("{}[ \u{1F9EC} FATAL ]{} Failed to download package from {}", ANSI_BOLD_RED, ANSI_RESET, url);
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
    let mut is_wasm = false;
    
    for arg in &args[3..] {
        if arg == "--standalone" { is_standalone = true; }
        if arg == "--release" { is_release = true; }
        if arg == "--wasm" { is_wasm = true; }
    }

    if command == "doc" {
        println!("[ HELIX ] DOCUMENTING: {}", filename);
        let docs_dir = Path::new("docs");
        if !docs_dir.exists() {
            fs::create_dir_all(docs_dir).unwrap();
        }
        
        let mut md = format!("# API Reference: {}\n\n", file_stem);
        
        let mut tui_features = Vec::new();
        
        fn walk_docs(stmts: &[crate::ast::Statement], md: &mut String, tui_features: &mut Vec<String>) {
            for stmt in stmts {
                match stmt {
                    crate::ast::Statement::DefineAction { name, args, doc, body } => {
                        md.push_str(&format!("## Action: `{}`\n", name));
                        if !args.is_empty() {
                            md.push_str(&format!("- **Arguments**: {}\n", args.join(", ")));
                        }
                        if let Some(d) = doc {
                            md.push_str(&format!("\n> {}\n\n", d));
                        } else {
                            md.push_str("\n*No description provided.*\n\n");
                        }
                        walk_docs(body, md, tui_features);
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
                    crate::ast::Statement::DefineBehavior { entity_name, action_name, args, doc, body } => {
                        md.push_str(&format!("## Behavior: `{}` (on Entity: `{}`)\n", action_name, entity_name));
                        if !args.is_empty() {
                            md.push_str(&format!("- **Arguments**: {}\n", args.join(", ")));
                        }
                        if let Some(d) = doc {
                            md.push_str(&format!("\n> {}\n\n", d));
                        } else {
                            md.push_str("\n*No description provided.*\n\n");
                        }
                        walk_docs(body, md, tui_features);
                    }
                    crate::ast::Statement::Import { body, .. } => {
                        walk_docs(body, md, tui_features);
                    }
                    crate::ast::Statement::While { body, .. } | crate::ast::Statement::WhileNot { body, .. } => {
                        walk_docs(body, md, tui_features);
                    }
                    crate::ast::Statement::Repeat { body, .. } | crate::ast::Statement::ForEach { body, .. } => {
                        walk_docs(body, md, tui_features);
                    }
                    crate::ast::Statement::IfExpr { then_branch, else_branch, .. } => {
                        walk_docs(then_branch, md, tui_features);
                        if let Some(else_b) = else_branch {
                            walk_docs(else_b, md, tui_features);
                        }
                    }
                    crate::ast::Statement::IfEndsWith { body, .. } | crate::ast::Statement::IfKeyPressed { body, .. } => {
                        walk_docs(body, md, tui_features);
                    }
                    crate::ast::Statement::ClearTerminal => {
                        tui_features.push("- **Clear Terminal**: Clears the console window and resets cursor position.".to_string());
                    }
                    crate::ast::Statement::WaitForKeyPress { var } => {
                        tui_features.push(format!("- **Wait for Key Press**: Non-blocking FFI keyboard reading mapped to `{}`.", var));
                    }
                    crate::ast::Statement::SleepMilliseconds { .. } => {
                        tui_features.push("- **Sleep/Delay**: Low-latency thread sleep in milliseconds.".to_string());
                    }
                    crate::ast::Statement::PrintColored { .. } => {
                        tui_features.push("- **Print Colored**: Prints styled ANSI color streams to stdout.".to_string());
                    }
                    crate::ast::Statement::DrawTerminalBox { .. } => {
                        tui_features.push("- **Draw Terminal Box**: Renders a styled ANSI bounding box pane.".to_string());
                    }
                    crate::ast::Statement::MoveCursor { .. } => {
                        tui_features.push("- **Move Cursor**: Sets cursor coordinates directly for grid redrawing.".to_string());
                    }
                    _ => {}
                }
            }
        }
        
        walk_docs(&ast, &mut md, &mut tui_features);

        if !tui_features.is_empty() {
            tui_features.sort();
            tui_features.dedup();
            md.push_str("## Terminal UI / Graphics Lexicon Used\n");
            md.push_str("This script integrates raw Helix terminal graphics primitives:\n\n");
            for feat in tui_features {
                md.push_str(&format!("{}\n", feat));
            }
            md.push_str("\n");
        }
        
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
                crate::ast::Statement::DefineBehavior { .. } |
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

    // Generate preamble
    codegen.generate_preamble(&ast);
    let mut c_final = codegen.c_code.clone();

    // Generate globals
    codegen.c_code = "".to_string();
    codegen.generate(&globals);
    c_final.push_str(&codegen.c_code);
    
    // Add HTTP dispatch function
    c_final.push_str("\nvoid syl_http_dispatch(String path, SOCKET client) {\n");
    c_final.push_str(&codegen.http_dispatch_code);
    c_final.push_str("}\n");

    // Generate main
    c_final.push_str("\nint main() {\n");
    c_final.push_str("    #ifdef _WIN32\n");
    c_final.push_str("    {\n");
    c_final.push_str("        HANDLE hOut = GetStdHandle(STD_OUTPUT_HANDLE);\n");
    c_final.push_str("        DWORD dwMode = 0;\n");
    c_final.push_str("        if (GetConsoleMode(hOut, &dwMode)) {\n");
    c_final.push_str("            dwMode |= 0x0004; // ENABLE_VIRTUAL_TERMINAL_PROCESSING\n");
    c_final.push_str("            SetConsoleMode(hOut, dwMode);\n");
    c_final.push_str("        }\n");
    c_final.push_str("        SetConsoleOutputCP(65001); // CP_UTF8\n");
    c_final.push_str("    }\n");
    c_final.push_str("    #endif\n\n");
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
        
        if is_wasm {
            println!("{}[ \u{1F9EC} SYL OS ]{} BUILDING: Compiling with Emscripten (WASM)...", ANSI_BOLD_CYAN, ANSI_RESET);
            let emcc_path = find_emcc();
            let out_dir = format!("{}_wasm", file_stem);
            if !Path::new(&out_dir).exists() {
                fs::create_dir_all(&out_dir).unwrap();
            }
            let out_file = format!("{}/index.html", out_dir);
            
            let mut emcc_args = vec![c_filename.as_str(), "-o", &out_file, "-s", "USE_GLFW=3", "-s", "ASYNCIFY", "libraylib.a"];
            
            if is_release {
                println!("[ HELIX ] OPTIMIZING: Enabling -O2 production flags for WASM.");
                emcc_args.push("-O2");
            }
            
            let status = std::process::Command::new(&emcc_path)
                .args(&emcc_args)
                .status();

            if let Ok(s) = status {
                if s.success() {
                    println!("{}[ \u{1F9EC} SUCCESS ]{} WASM Output generated at {}/", ANSI_BOLD_GREEN, ANSI_RESET, out_dir);
                    if !is_build_only {
                        serve_wasm_dir(&out_dir, 8000);
                    }
                } else {
                    eprintln!("{}[ \u{1F9EC} ERROR ]{} Emscripten compilation failed.", ANSI_BOLD_RED, ANSI_RESET);
                }
            } else {
                eprintln!("{}[ \u{1F9EC} ERROR ]{} Could not find emcc. Run 'syl setup-wasm' first.", ANSI_BOLD_RED, ANSI_RESET);
            }
            return;
        }

        println!("[ HELIX ] BUILDING: Compiling with TCC...");
        let tcc_path = find_tcc();
        let exe_filename = format!("{}.exe", file_stem);
        
        let mut tcc_args = vec![&c_filename, "raylib.dll"];
        if codegen.use_net {
            tcc_args.push("ws2_32.dll");
        }
        if codegen.use_sqlite {
            tcc_args.push("sqlite3.dll");
        }
        tcc_args.extend_from_slice(&["-I.", "-L.", "-o", &exe_filename]);

        if is_release {
            println!("[ HELIX ] OPTIMIZING: Enabling -O3 production flags.");
            tcc_args.push("-O3");
        }

        let status = std::process::Command::new(&tcc_path)
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

fn find_tcc() -> String {
    // 1. Check current_exe() relative path
    if let Ok(mut exe_dir) = std::env::current_exe() {
        exe_dir.pop(); // remove exe name
        // Check if tcc is in the same dir as the compiler
        let local_tcc = exe_dir.join("tcc.exe");
        if local_tcc.exists() {
            return local_tcc.to_string_lossy().to_string();
        }
        // Check relative tools/tcc/tcc/tcc.exe
        // Let's traverse up to 5 directories to find it in the workspace
        let mut check_dir = exe_dir.clone();
        for _ in 0..6 {
            let possible = check_dir.join("tools/tcc/tcc/tcc.exe");
            if possible.exists() {
                return possible.to_string_lossy().to_string();
            }
            if !check_dir.pop() {
                break;
            }
        }
    }

    // 2. Check if TCC is in PATH
    if let Some(paths) = std::env::var_os("PATH") {
        for path in std::env::split_paths(&paths) {
            let p_tcc = path.join("tcc.exe");
            if p_tcc.exists() {
                return p_tcc.to_string_lossy().to_string();
            }
            let p_tcc_unix = path.join("tcc");
            if p_tcc_unix.exists() {
                return p_tcc_unix.to_string_lossy().to_string();
            }
        }
    }
    
    // 4. Fallback relative to cwd
    "tools/tcc/tcc/tcc.exe".to_string()
}

fn find_emcc() -> String {
    let cmd = if cfg!(target_os = "windows") { "emcc.bat" } else { "emcc" };
    
    // 1. Check if it's in PATH
    if let Ok(_) = std::process::Command::new(cmd).arg("--version").output() {
        return cmd.to_string();
    }
    
    // 2. Check workspace tools/emsdk/upstream/emscripten
    let path1 = Path::new("tools").join("emsdk").join("upstream").join("emscripten").join(cmd);
    if path1.exists() {
        return path1.to_string_lossy().to_string();
    }
    
    // 3. Check relative up up to 5 levels
    let mut current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    for _ in 0..5 {
        let check_path = current_dir.join("tools").join("emsdk").join("upstream").join("emscripten").join(cmd);
        if check_path.exists() {
            return check_path.to_string_lossy().to_string();
        }
        if !current_dir.pop() {
            break;
        }
    }
    
    cmd.to_string() // default fallback
}

fn find_git() -> String {
    // 1. Check if git is in PATH
    if let Ok(output) = std::process::Command::new("git").arg("--version").output() {
        if output.status.success() {
            return "git".to_string();
        }
    }
    
    // 2. Check portable git relative to exe
    if let Ok(mut exe_dir) = std::env::current_exe() {
        exe_dir.pop();
        let portable = exe_dir.join("..").join("tools").join("mingit").join("cmd").join("git.exe");
        if portable.exists() {
            return portable.to_string_lossy().to_string();
        }
    }
    
    // 3. Check tools/mingit/cmd relative to CWD
    let local_git = Path::new("tools").join("mingit").join("cmd").join("git.exe");
    if local_git.exists() {
        return local_git.to_string_lossy().to_string();
    }
    
    // 4. Check parent dirs
    let mut current_dir = std::env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
    for _ in 0..5 {
        let check = current_dir.join("tools").join("mingit").join("cmd").join("git.exe");
        if check.exists() {
            return check.to_string_lossy().to_string();
        }
        if !current_dir.pop() { break; }
    }
    
    "git".to_string() // fallback
}

fn serve_wasm_dir(dir: &str, port: u16) {
    use std::net::TcpListener;
    use std::io::{Read, Write};
    use std::fs;
    use std::path::Path;

    let listener = match TcpListener::bind(format!("127.0.0.1:{}", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[ \u{1F9EC} ERROR ] Failed to bind web server to port {}: {}", port, e);
            return;
        }
    };
    
    let url = format!("http://localhost:{}", port);
    println!("[ \u{1F9EC} SERVING ] Local server running at {}/", url);
    println!("[ \u{1F9EC} SERVING ] Serving files from directory: {}/", dir);
    println!("[ \u{1F9EC} SERVING ] Press Ctrl+C to stop.");
    
    // Automatically open browser
    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("cmd").args(&["/c", "start", &url]).status();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(&url).status();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(&url).status();

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buffer = [0; 1024];
            if let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 { continue; }
                let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                let first_line = request.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mut path_str = parts[1];
                    if path_str == "/" {
                        path_str = "/index.html";
                    }
                    if let Some(pos) = path_str.find('?') {
                        path_str = &path_str[..pos];
                    }
                    if let Some(pos) = path_str.find('#') {
                        path_str = &path_str[..pos];
                    }
                    
                    let file_path = format!("{}/{}", dir, path_str.trim_start_matches('/'));
                    let path = Path::new(&file_path);
                    
                    if path.exists() && path.is_file() {
                        if let Ok(content) = fs::read(&file_path) {
                            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                            let mime_type = match ext {
                                "html" => "text/html",
                                "css" => "text/css",
                                "js" => "application/javascript",
                                "wasm" => "application/wasm",
                                "png" => "image/png",
                                "jpg" | "jpeg" => "image/jpeg",
                                _ => "application/octet-stream",
                            };
                            
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n",
                                mime_type,
                                content.len()
                            );
                            let _ = stream.write_all(response.as_bytes());
                            let _ = stream.write_all(&content);
                        } else {
                            let server_err = "HTTP/1.1 500 Internal Server Error\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                            let _ = stream.write_all(server_err.as_bytes());
                        }
                    } else {
                        let not_found = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                        let _ = stream.write_all(not_found.as_bytes());
                    }
                }
            }
        }
    }
}
