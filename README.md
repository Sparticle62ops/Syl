# Syl Programming Language [v2.5.0]

**Syl** is a naturalistic systems programming language that compiles English-like syntax to native binaries via C transpilation. Write readable code, get real executables.

## Features

- **Natural English Syntax** — Write code that reads like prose: `Set score to 0.`, `If w key is pressed:`
- **Graphical Games** — Built-in Raylib integration for windowed graphical applications
- **Entity System** — Define entities with fields and behaviors for OOP-style programming
- **Helix IR** — Intermediate representation for analysis and optimization
- **C Transpilation** — Generates portable C code compiled via TCC
- **Arena Memory** — Frame-based arena allocator with automatic per-frame resets

## Quick Start

### 1. Install Dependencies
```powershell
# Downloads TCC, Raylib, Git, and Rust toolchain
powershell -ExecutionPolicy Bypass -File install_syl.ps1
```

### 2. Build the Compiler
```powershell
cargo build
```

### 3. Compile & Run Examples
```powershell
# Build the Snake game (graphical window)
./target/debug/syl.exe build examples/snake.syl

# Build the Endless Shooter game
./target/debug/syl.exe build examples/web_game.syl

# Build the Bouncing Ball physics sim
./target/debug/syl.exe build examples/bouncing_ball.syl

# Run the compiled game
./snake.exe
```

## Example Games

### Snake (`examples/snake.syl`)
Classic snake game with graphical window, score tracking, growing tail, and restart support. Controls: WASD to move, R to restart.

### Endless Alien Shooter (`examples/web_game.syl`)
Top-down shooter with player movement, bullet firing, enemy AI that tracks the player, collision detection, HP system, and score. Controls: WASD to move, SPACE to shoot, R to restart.

### Bouncing Ball (`examples/bouncing_ball.syl`)
Physics simulation with a bouncing ball and gravity.

## Language Sample

```text
Set score to 0.
Set lives to 3.

Please open a window with width 800 and height 600 titled "My Game".

Repeat while the window is not closing:
    Begin drawing.
    Clear the background to "black".
    If w key is pressed:
        Set player_y to (player_y - 5).
    Draw circle at player_x, player_y with radius 20 colored "green".
    Draw text "Score: " joined with score at position 10, 10.
    End drawing.
```

## Entity & Behavior System

```text
Define an entity called Player:
    Set x to 400.
    Set y to 300.
    Set hp to 100.

Define behavior move_right for Player taking speed:
    Set x of self to (x of self + speed).

Create Player called hero.
Trigger move_right on hero taking 5.
```

## Architecture

```
source.syl → [Lexer] → [Parser] → [AST] → [Helix IR] → [C Codegen] → [TCC] → native .exe
```

| Component | File | Purpose |
|-----------|------|---------|
| Lexer | `src/lexer.rs` | Tokenization with indentation tracking |
| Parser | `src/parser.rs` | Recursive descent parser producing AST |
| AST | `src/ast.rs` | Statement and expression types |
| IR | `src/ir.rs` | Helix intermediate representation |
| Codegen | `src/codegen.rs` | C code generation with Raylib bindings |
| Main | `src/main.rs` | CLI, build pipeline, TCC invocation |

## Project Structure

```
syl/
├── src/                    # Compiler source (Rust)
│   ├── main.rs             # CLI and build pipeline
│   ├── lexer.rs            # Tokenizer
│   ├── parser.rs           # Parser
│   ├── ast.rs              # AST definitions
│   ├── ir.rs               # Helix IR
│   ├── codegen.rs          # C transpiler
│   ├── tui.rs              # Terminal UI (Syl Studio)
│   └── emit_native.rs      # Native emission helpers
├── examples/               # Example .syl programs
│   ├── snake.syl           # Graphical snake game
│   ├── web_game.syl        # Endless alien shooter
│   ├── bouncing_ball.syl   # Physics simulation
│   ├── fibonacci.syl       # Fibonacci sequence
│   └── ...
├── docs/                   # Language documentation
├── lib/                    # TCC compiler (portable)
├── Cargo.toml              # Rust project manifest
└── README.md
```

## License

MIT

---
*Syl v2.5.0 — Natural Language, Native Performance*
