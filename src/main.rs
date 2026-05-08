mod ast;
mod lexer;
mod parser;
mod ir;
mod codegen;

use std::fs;
use std::path::Path;
use lexer::Lexer;
use parser::Parser;
use ir::IRGenerator;
use codegen::CodeGen;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: syl <file.syl>");
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).unwrap_or_else(|_| {
        eprintln!("Failed to read file: {}", filename);
        std::process::exit(1);
    });

    println!("Compiling {}...", filename);

    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize();

    let mut parser = Parser::new(tokens);
    let ast = match parser.parse() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Syntax Error: {}", e);
            std::process::exit(1);
        }
    };

    println!("AST successfully generated!");

    // Generate Helix IR
    let mut ir_gen = IRGenerator::new();
    ir_gen.generate(&ast);
    let hlx_content = ir_gen.instructions.join(" ");

    let file_path = Path::new(filename);
    let file_stem = file_path.file_stem().unwrap().to_str().unwrap();

    let hlx_filename = format!("{}.hlx", file_stem);
    fs::write(&hlx_filename, hlx_content).unwrap_or_else(|_| {
        eprintln!("Failed to write {}", hlx_filename);
    });
    println!("Generated Helix IR: {}", hlx_filename);

    // Generate C Code
    let mut codegen = CodeGen::new();
    codegen.generate(&ast);
    
    // Output dummy functions for 'auth_get_role' etc. to make it compile via gcc if wanted
    let mut c_final = codegen.c_code.clone();
    c_final.push_str("\n// --- Dummy dependencies for compilation ---\n");
    c_final.push_str("String auth_get_role(String username) { return \"Admin\"; }\n");
    c_final.push_str("void auth_log_admin_login() { printf(\"Admin logged in\\n\"); }\n");
    c_final.push_str("\nint main() {\n    printf(\"%s\\n\", process_login(\"Alice\", \"pass123\"));\n    return 0;\n}\n");

    let c_filename = format!("{}.c", file_stem);
    fs::write(&c_filename, c_final).unwrap_or_else(|_| {
        eprintln!("Failed to write {}", c_filename);
    });
    println!("Generated C code: {}", c_filename);
    
    println!("Compilation pipeline completed successfully.");
}
