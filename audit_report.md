# Syl v1.0 -- Principal Architectural Audit Report

> Auditor: Senior Principal Language Architect
> Date: 2026-05-12
> Scope: Full codebase review of `src/` (ast.rs, lexer.rs, parser.rs, ir.rs, codegen.rs, emit_native.rs, main.rs)

---

## SECTION 1: CRITICAL VULNERABILITIES (Compilation Blockers)

> [!CAUTION]
> The following bugs **prevent the compiler from compiling at all**. The codebase as shipped cannot produce a working `syl.exe`.

### BUG-001: Duplicate `parse_set` Method (FATAL)
- **File:** [parser.rs](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/src/parser.rs)
- **Lines:** 256-277 AND 313-335
- **Severity:** FATAL -- Rust will refuse to compile a struct with two methods of the same name.
- **Cause:** The v0.9.5 `parse_set` (with Entity field support) was added without removing the original v0.2 `parse_set` (with GetItem support).
- **Fix:** Merge both into a single method that handles plain assignment, entity field access, AND GetItem.

### BUG-002: Broken Metadata Parser -- "The" Stripped by Lexer (FATAL)
- **File:** [lexer.rs](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/src/lexer.rs#L135-L140)
- **Severity:** FATAL -- The entire v1.0 Project Manifest feature is dead code.
- **Cause:** The lexer strips "the" as a filler word using **case-insensitive** matching (`eq_ignore_ascii_case`). This means "The" at the start of `The project is named "Sentinel".` is silently consumed and never reaches the parser. The metadata parser at lines 73-114 can never see "The".
- **Fix:** Change filler comparison to case-sensitive (`==`). Lowercase "the" is still stripped; capitalized "The" passes through.

### BUG-003: IR Generator Has Non-Exhaustive Match (FATAL)
- **File:** [ir.rs](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/src/ir.rs#L45-L170)
- **Severity:** FATAL -- Rust compile error due to missing match arms.
- **Detail:** `gen_statement` only handles 20 of the 35+ Statement variants. Missing: `CallAction`, `WhileNot`, `IfKeyPressed`, `GetItem`, `SetItem`, `Increase`, `CreateList`, `AddToList`, `Execute`, `DefineEntity`, `CreateEntity`, `SetField`, `Download`, `ListWords`.
- **Same issue in `gen_expr`:** Missing `GetField`, `Join`, `Sqrt`, `Pow`, `Random`, `FileSize`.
- **Fix:** Add wildcard `_` arms or implement proper IR emission for each variant.

### BUG-004: `emit_native.rs` Missing Import (FATAL)
- **File:** [emit_native.rs](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/src/emit_native.rs#L4)
- **Severity:** FATAL -- Uses `Expr::Join` at line 66 but only imports `Statement`.
- **Fix:** Add `use crate::ast::Expr;`

### BUG-005: Cranelift `types::B1` Removed (FATAL)
- **File:** [emit_native.rs](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/src/emit_native.rs#L33)
- **Severity:** FATAL -- `types::B1` was removed from Cranelift. Compilation fails.
- **Fix:** Replace with `types::I8`.

---

## SECTION 2: SECURITY VULNERABILITIES (Helix Guard Analysis)

### VULN-001: Use-After-Free via Pointer Alias
The `Give` statement sets the source to `NULL` in C, but if a pointer alias was created earlier (via `Copy` or by passing to a function), the alias remains valid and points to the original memory. The Helix Guard's parity tagging only tracks the **variable name**, not the **underlying pointer**.

### VULN-002: Buffer Overflow in Dynamic Lists
All lists are allocated with a fixed `malloc(1024 * sizeof(String))`. Writing more than 1024 items causes a **heap buffer overflow** with no guard check.

### VULN-003: Memory Leaks in String Joining
`Expr::Join` allocates via `malloc` but **never frees**. In a loop, this is an unbounded memory leak. No deallocation strategy exists.

### VULN-004: Unchecked `stat()` Return in FileSize
The `FileSize` codegen calls `stat()` without checking the return value. If the file doesn't exist, `st_size` contains garbage data.

### VULN-005: Shell Injection in Download
`Download` uses `sprintf` + `system("curl ...")`. A malicious URL containing shell metacharacters (`;rm -rf /`) would execute arbitrary commands.

---

## SECTION 3: PERFORMANCE BOTTLENECKS

| Issue | Location | Impact |
|-------|----------|--------|
| `pending_tokens.remove(0)` is O(n) | lexer.rs:50 | Quadratic tokenization on deeply indented files |
| String joins leak memory in loops | codegen.rs:361-364 | Unbounded heap growth |
| Fixed 1024-item list cap | codegen.rs:170-171 | Hard crash on large datasets |
| `clone()` on every token peek | parser.rs:130 | Unnecessary allocations in hot path |
| Recursive module import | parser.rs:188-211 | No cycle detection, potential stack overflow |

---

## SECTION 4: LEXICAL AMBIGUITY

| Pattern | Collision Risk | Status |
|---------|---------------|--------|
| `Set X to Y to the power of Z.` | "to" is both assignment and power-of | Handled (backtracking) |
| `Move file to folder.` vs `Move file to "Archive".` | "to" as keyword vs preposition | Safe (parse_expr resolves) |
| `Print name of user.` | "of" consumed by GetField | Risk: "name" becomes field, "user" becomes instance |
| `Add item and count to list.` | "and" greedily consumed | Risk: "count" eaten as second arg |

---

## SECTION 5: STANDARD LIBRARY GAPS

| Module | Missing | Priority |
|--------|---------|----------|
| `str` | split, trim, replace, length, contains | CRITICAL |
| `time` | current time, sleep, date formatting | HIGH |
| `json` | parse, stringify | HIGH |
| `crypto` | hash (SHA-256), random bytes | MEDIUM |
| `io` | stdin input, line-by-line reading | CRITICAL |
| `math` | abs, min, max, floor, ceil | HIGH |
| `sys` | environment variables, command args | HIGH |

---

## SECTION 6: THE SYL 2.0 ROADMAP

1. **Type System:** Introduce `Number`, `Text`, `Boolean`, `List of [Type]` to enable compile-time type checking
2. **Arena Allocator:** Replace per-string `malloc` with a per-scope arena that frees all allocations at scope exit
3. **Proper Ownership Model:** Track ownership at the IR level, not just variable names -- use a borrow graph
4. **Incremental Compilation:** Cache parsed modules to avoid re-parsing on every build
5. **Error Recovery:** Parser should not abort on first error -- collect multiple diagnostics
6. **WASM Backend:** Add a WebAssembly target alongside Cranelift native and C-transpilation
7. **Package Manager:** `Bring in "github.com/user/package" as pkg.` with version resolution
