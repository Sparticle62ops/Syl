# Syl Compiler [v0.3.0-alpha]

Syl is a high-performance, naturalistic systems programming language designed for industrial-grade desktop applications and systems tooling. It combines the readability of natural English with the performance of native machine code.

## Key Features

- **Naturalistic Syntax:** Code that reads like documentation (e.g., `Set the count to 0.`).
- **Helix Safety Guard:** Runtime memory parity checks (Even/Odd tagging) to prevent use-after-free and moved-variable violations.
- **Native Execution:** Compiles directly to native binaries using a high-performance Cranelift-based backend (with C-transpilation fallback).
- **Interactive Graphics:** Built-in FFI bridge to Raylib for high-performance 2D/3D graphics and interaction.
- **Portable Toolchain:** Integrated support for Tiny C Compiler (TCC), Zig (Linker), and portable Rust environments.

## Quick Start

### Prerequisites
Syl requires a portable C compiler (TCC) and Raylib binaries for its studio launcher. Run the bootstrap script to set up your environment:

```powershell
./launch_syl.ps1
```

### Running the Counter Test
To see the Syl interaction engine in action:

```powershell
# Requires Rust toolchain
cargo run -- run counter_test.syl
```

## Language Sample

```text
Bring in "ui.syl" as ui.

Set the count to 0.
Please open a window with width 400 and height 300 titled "Syl Counter".

Repeat while the window is not closing:
    If the Space key is pressed:
        Increase the count by 1.
    
    Begin drawing.
    Clear the background to black.
    Draw text "Count: " and the count at position 50, 100.
    End drawing.
```

## Versioning
This is **v0.3.0-alpha**, focusing on native interaction and industrial CLI UX.
