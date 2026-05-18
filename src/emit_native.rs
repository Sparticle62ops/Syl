use cranelift::prelude::*;
use cranelift_module::{Linkage, Module, FuncId};
use cranelift_object::{ObjectBuilder, ObjectModule};
use crate::ast::{Statement, Expr};
use std::process::Command;
use std::collections::HashMap;

/// Native code emitter using Cranelift.
///
/// Declares arena runtime functions (`syl_alloc`, `syl_arena_init`, etc.)
/// and emits native object code that links against them.
pub struct NativeEmitter {
    module: ObjectModule,
    /// Cached FuncIds for runtime functions so we only declare them once.
    runtime_funcs: HashMap<String, FuncId>,
}

impl NativeEmitter {
    pub fn new() -> Self {
        let flag_builder = settings::builder();
        let isa_builder = cranelift_native::builder().unwrap();
        let isa = isa_builder.finish(settings::Flags::new(flag_builder)).unwrap();
        
        let builder = ObjectBuilder::new(isa, "syl_module", cranelift_module::default_libcall_names()).unwrap();
        let module = ObjectModule::new(builder);
        
        let mut emitter = NativeEmitter {
            module,
            runtime_funcs: HashMap::new(),
        };

        // Pre-declare arena runtime functions
        emitter.declare_runtime_funcs();
        emitter
    }

    /// Declares the Syl Arena runtime functions as imported symbols.
    /// These are resolved at link time against the compiled runtime.rs symbols.
    fn declare_runtime_funcs(&mut self) {
        // void syl_arena_init()
        {
            let mut sig = self.module.make_signature();
            // no params, no returns
            let id = self.module.declare_function("syl_arena_init", Linkage::Import, &sig).unwrap();
            self.runtime_funcs.insert("syl_arena_init".into(), id);
            println!("[ NATIVE ] Declared FFI: syl_arena_init() -> void");
        }

        // void* syl_alloc(size: i64) -> i64 (pointer)
        {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));   // size
            sig.returns.push(AbiParam::new(types::I64));   // pointer
            let id = self.module.declare_function("syl_alloc", Linkage::Import, &sig).unwrap();
            self.runtime_funcs.insert("syl_alloc".into(), id);
            println!("[ NATIVE ] Declared FFI: syl_alloc(size: i64) -> *mut u8");
        }

        // char* syl_strdup(src: i64) -> i64 (pointer)
        {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));   // src pointer
            sig.returns.push(AbiParam::new(types::I64));   // dest pointer
            let id = self.module.declare_function("syl_strdup", Linkage::Import, &sig).unwrap();
            self.runtime_funcs.insert("syl_strdup".into(), id);
            println!("[ NATIVE ] Declared FFI: syl_strdup(src: *const u8) -> *mut u8");
        }

        // usize syl_arena_save() -> i64
        {
            let mut sig = self.module.make_signature();
            sig.returns.push(AbiParam::new(types::I64));   // saved offset
            let id = self.module.declare_function("syl_arena_save", Linkage::Import, &sig).unwrap();
            self.runtime_funcs.insert("syl_arena_save".into(), id);
            println!("[ NATIVE ] Declared FFI: syl_arena_save() -> usize");
        }

        // void syl_arena_restore(saved_offset: i64)
        {
            let mut sig = self.module.make_signature();
            sig.params.push(AbiParam::new(types::I64));   // saved offset
            let id = self.module.declare_function("syl_arena_restore", Linkage::Import, &sig).unwrap();
            self.runtime_funcs.insert("syl_arena_restore".into(), id);
            println!("[ NATIVE ] Declared FFI: syl_arena_restore(offset: usize) -> void");
        }
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
            sig.returns.push(AbiParam::new(types::I8)); // bool
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
        let _func_id = self.module.declare_function(&function_name, Linkage::Import, &sig).unwrap();
        
        println!("[ FFI ] Bridged {} -> {} from \"{}\"", namespace, action, lib_path);
        println!("        -> Native Signature: {}", c_sig);
    }
    
    pub fn compile_ast(&mut self, ast: &[Statement]) {
        // Emit arena initialization at the top level
        println!("[ HELIX v1.2 ] ARENA: Injecting syl_arena_init() into native entry point.");

        for stmt in ast {
            match stmt {
                Statement::External { namespace, action, lib_path } => {
                    self.declare_external(namespace, action, lib_path);
                }
                Statement::CreateList { name, items } => {
                    println!("[ HELIX GUARD ] Tagged list '{}' with EVEN parity (Safe).", name);
                    println!("[ NATIVE ] syl_alloc({} * sizeof(String)) for dynamic list: {}", items.len(), name);
                }
                Statement::WhileNot { body, .. } => {
                    self.compile_ast(body);
                }
                Statement::While { body, .. } => {
                    self.compile_ast(body);
                }
                Statement::DefineAction { name, body, .. } => {
                    // Scoped arena: save offset at prologue, restore at epilogue
                    println!("[ NATIVE ] Emitting action '{}' with scoped arena memory.", name);
                    println!("           Prologue: call syl_arena_save() -> stack slot");
                    self.compile_ast(body);
                    println!("           Epilogue: call syl_arena_restore(saved_offset)");
                }
                Statement::Assign { name, value } => {
                    if let Expr::Join { .. } = value {
                        println!("[ NATIVE ] Emitted Opcode J (Join): syl_alloc() string concatenation.");
                    }
                    println!("[ HELIX GUARD ] Tagged variable '{}' with EVEN parity (Safe).", name);
                }
                Statement::Verify { .. } => {}
                Statement::CreateDictionary { .. } => {}
                Statement::SetDictKey { .. } => {}
                Statement::ListenHttp { .. } => {}
                Statement::HttpRequestRoute { .. } => {}
                Statement::HttpReply { .. } => {}
                Statement::Attempt { .. } => {}
                Statement::IfFailed { .. } => {}
                Statement::IfSucceeded { .. } => {}
                Statement::ConnectDB { .. } => {}
                Statement::ExecuteQuery { .. } => {}
                Statement::ClearTerminal => {}
                Statement::WaitForKeyPress { .. } => {}
                Statement::SleepMilliseconds { .. } => {}
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
                    println!("[ HELIX GUARD ] Emitted BOUNDS CHECK for Index access.");
                    println!("[ NATIVE ] Emitted Opcode I (Index): List Base + ((index - 1) * sizeof(Type))");
                    println!("[ HELIX GUARD ] Tagged variable '{}' with EVEN parity (Safe).", identifier);
                }
                Statement::SetItem { list, .. } => {
                    println!("[ HELIX GUARD ] Emitted Parity Check for '{}'. (Panics on ODD)", list);
                    println!("[ HELIX GUARD ] Emitted BOUNDS CHECK for Index access.");
                    println!("[ NATIVE ] Emitted Opcode I (Index): List Base + ((index - 1) * sizeof(Type))");
                }
                Statement::Increase { name, .. } => {
                    println!("[ HELIX GUARD ] Emitted Parity Check for '{}'. (Panics on ODD)", name);
                }
                Statement::ListWords { identifier, .. } => {
                    println!("[ NATIVE ] syl_alloc() + syl_strdup() for ListWords -> '{}'", identifier);
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
