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
    let is_standalone = args.len() > 3 && args[3] == "--standalone";
    let source = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file: {}", filename);
        std::process::exit(1);
    });

    println!("Syl Compiler [v{}-Release]", VERSION);
    println!("[ HELIX ] PARSING: {}", filename);

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
    
    // Wrap top-level statements in main()
    let mut c_final = codegen.c_code.clone();
    c_final.push_str("\nint main() {\n");
    c_final.push_str("    syl_arena_init(1024 * 1024 * 10); // 10MB default arena\n");
    codegen.c_code = "".to_string(); // reset for generation
    codegen.generate(&ast);
    c_final.push_str(&codegen.c_code);
    c_final.push_str("\n    return 0;\n}\n");

    let c_filename = format!("{}.c", file_stem);
    fs::write(&c_filename, c_final).unwrap_or_else(|_| {
        eprintln!("[ HELIX ] ERROR: Failed to write {}", c_filename);
    });
    println!("[ HELIX ] TRANSPILING: {}", c_filename);

    if command == "run" {
        println!("[ HELIX ] BUILDING: Compiling with TCC...");
        let tcc_path = "tools/tcc/tcc/tcc.exe";
        let exe_filename = format!("{}.exe", file_stem);
        
        let status = std::process::Command::new(tcc_path)
            .args(&[&c_filename, "raylib.dll", "-I.", "-L.", "-o", &exe_filename])
            .status();

        if let Ok(s) = status {
            if s.success() {
                println!("[ HELIX ] SUCCESS: Binary \"{}\" generated.", exe_filename);
                println!("[ HELIX ] RUNNING: Executing {}...", exe_filename);
                let _ = std::process::Command::new(format!("./{}", exe_filename)).status();
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
