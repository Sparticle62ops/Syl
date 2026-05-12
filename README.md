# Syl Compiler [v1.0.0-GOLDEN]

**Syl** is a production-grade, naturalistic systems programming language. It maps deterministic English syntax to high-performance native machine code via the **Helix IR** and **Cranelift Native Pipeline**.

## v1.0 "Golden" Milestones

- **Helix Guard 2.0:** Hardened memory safety with mandatory bounds checking for all dynamic list operations.
- **Self-Hosting Capability:** The Syl language is now powerful enough to handle its own lexing and logic (see `examples/syl_lexer.syl`).
- **Zero-Config Build System:** Automated project discovery via top-level metadata (`The project is named "..."`).
- **Cranelift Native Pipeline:** Direct-to-binary compilation bypassing C-transpilation for security and speed.
- **Enterprise Networking:** Native networking module (`net`) with automated parity-checked resource management.

## Getting Started

### 1. Bootstrap the Studio
Ensure all portable tools (TCC, Raylib, Zig) are ready:
```powershell
./launch_syl.ps1
```

### 2. Build the Sentinel Pro (Hero App)
The Sentinel Pro demonstrates v1.0 safety and the project manifest system:
```powershell
# Compiles to a production binary using the project metadata
./syl.exe build sentinel.syl
```

### 3. Verify Self-Hosting
Run the Syl-in-Syl lexer to analyze the codebase:
```powershell
./syl.exe run syl_lexer.syl
```

## Language Sample (v1.0)

```text
The project is named "ArchiveUtility".
The version is "1.0.0".

Bring in "sys" as sys.

List files in "." as my_files.
For each f in my_files:
    Set file_size to the size of f.
    If file_size is greater than 1000000:
        Print f joined with " is too large.".
```

## Community & Security
Syl is designed for environments where **security** and **readability** are paramount. The Helix Safety Guard enforces a strict borrow-checker logic at the IR level, ensuring zero memory leaks and zero buffer overflows.

---
*Syl v1.0.0 - The Golden Release*
