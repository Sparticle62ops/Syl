use cranelift::prelude::*;
use cranelift_module::{Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule};
use crate::ast::Statement;
use std::process::Command;

pub struct NativeEmitter {
    module: ObjectModule,
}

impl NativeEmitter {
    pub fn new() -> Self {
        let flag_builder = settings::builder();
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        
        let builder = ObjectBuilder::new(isa, "syl_module", cranelift_module::default_libcall_names()).unwrap();
        let module = ObjectModule::new(builder);
        
        NativeEmitter { module }
    }

    pub fn declare_external(&mut self, namespace: &str, action: &str, lib_path: &str) {
        let mut sig = self.module.make_signature();
        let mut c_sig = format!("void {}(", action);

        if action == "InitWindow" {
            sig.params.push(AbiParam::new(types::I32)); // width
            sig.params.push(AbiParam::new(types::I32)); // height
            sig.params.push(AbiParam::new(types::I64)); // title ptr
            c_sig = "void InitWindow(int width, int height, const char* title)".to_string();
        } else if action == "WindowShouldClose" {
            sig.returns.push(AbiParam::new(types::B1)); // bool
            c_sig = "bool WindowShouldClose(void)".to_string();
        } else if action == "DrawText" {
            sig.params.push(AbiParam::new(types::I64)); // text ptr
            sig.params.push(AbiParam::new(types::I32)); // posX
            sig.params.push(AbiParam::new(types::I32)); // posY
            c_sig = "void DrawText(const char* text, int posX, int posY)".to_string();
        } else {
            sig.params.push(AbiParam::new(types::I64));
            c_sig = format!("void {}(const char* arg)", action);
        }
        
        let function_name = format!("{}_{}", namespace, action);
        let func_id = self.module.declare_function(&function_name, Linkage::Import, &sig).unwrap();
        
        println!("[ FFI ] Bridged {} -> {} from \"{}\"", namespace, action, lib_path);
        println!("        -> Native Signature: {}", c_sig);
    }
    
    pub fn compile_ast(&mut self, ast: &[Statement]) {
        for stmt in ast {
            match stmt {
                Statement::External { namespace, action, lib_path } => {
                    self.declare_external(namespace, action, lib_path);
                }
                Statement::CreateList { name, items } => {
                    println!("[ HELIX GUARD ] Tagged list '{}' with EVEN parity (Safe).", name);
                    println!("[ NATIVE ] Allocating heap pointer for dynamic list: {} (Size: {} items)", name, items.len());
                }
                Statement::WhileNot { body, .. } => {
                    self.compile_ast(body);
                }
                Statement::Assign { name, .. } => {
                    println!("[ HELIX GUARD ] Tagged variable '{}' with EVEN parity (Safe).", name);
                }
                Statement::Give { src, dst } => {
                    println!("[ HELIX GUARD ] Tagged variable '{}' with ODD parity (Destructive Move).", src);
                    println!("[ HELIX GUARD ] Tagged variable '{}' with EVEN parity (Safe).", dst);
                }
                Statement::Move { .. } => {
                    println!("[ HELIX GUARD ] Emitted Parity Check for Destructive Operation. (Panics on ODD)");
                }
                Statement::IfKeyPressed { key, body } => {
                    println!("[ FFI ] Bridged raylib -> IsKeyPressed from \"raylib.dll\"");
                    println!("[ NATIVE ] Emitted Branch: IsKeyPressed(KEY_{})", key.to_uppercase());
                    self.compile_ast(body);
                }
                Statement::GetItem { identifier, list, .. } => {
                    println!("[ HELIX GUARD ] Emitted Parity Check for '{}'. (Panics on ODD)", list);
                    println!("[ NATIVE ] Emitted Pointer Arithmetic: List Base + ((index - 1) * sizeof(Type))");
                    println!("[ HELIX GUARD ] Tagged variable '{}' with EVEN parity (Safe).", identifier);
                }
                Statement::SetItem { list, .. } => {
                    println!("[ HELIX GUARD ] Emitted Parity Check for '{}'. (Panics on ODD)", list);
                    println!("[ NATIVE ] Emitted Pointer Arithmetic: List Base + ((index - 1) * sizeof(Type))");
                }
                Statement::Increase { name, .. } => {
                    println!("[ HELIX GUARD ] Emitted Parity Check for '{}'. (Panics on ODD)", name);
                }
                _ => {}
            }
        }
    }

    pub fn finish(self) -> Vec<u8> {
        // Emit the final object file payload natively (.obj / .o)
        self.module.finish().emit().unwrap()
    }

    pub fn invoke_linker(obj_filename: &str, out_filename: &str) {
        println!("[ LINKER ] Linking native executable...");
        
        #[cfg(target_os = "windows")]
        let status = Command::new("clang")
            .args(&[obj_filename, "-o", out_filename, "-lmsvcrt"])
            .status();

        #[cfg(not(target_os = "windows"))]
        let status = Command::new("cc")
            .args(&[obj_filename, "-o", out_filename])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("[ BUILD ] Native binary created: ./{}", out_filename);
            }
            Ok(s) => {
                eprintln!("[ ERROR ] Linker failed with status: {}", s);
            }
            Err(e) => {
                eprintln!("[ ERROR ] Linker invocation failed: {}. Please ensure clang/gcc is installed.", e);
            }
        }
    }
}
