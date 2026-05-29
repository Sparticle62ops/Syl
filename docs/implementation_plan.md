# Fix Shooter Game Memory Leak and Convert Snake Game to Raylib GUI

## User Review Required

> [!IMPORTANT]
> **Game loop memory leak fix**: We will modify the C transpiler in `src/codegen.rs` to automatically save the arena allocator's offset at the start of the `while (!WindowShouldClose())` game loop and restore it at the end of each frame. This prevents dynamic string allocations (like scoreboard formatting) from overflowing the 10MB arena and crashing/scrambling the game.
> 
> **Snake Game Refactoring**: We are refactoring `examples/snake.syl` from a CLI/TUI application to a graphical windowed Raylib application. The gameplay will use a 2D grid, feature a growing snake using lists, maintain a score, and handle input via `W/A/S/D` keys smoothly.

## Proposed Changes

### Component 1: Compiler Codegen

#### [MODIFY] [codegen.rs](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/src/codegen.rs)
- Modify `Statement::WhileNot` translation to detect `c_cond == "WindowShouldClose"`.
- If it is the game loop, insert `size_t _frame_arena_save = global_arena.offset;` inside the loop body before other statements, and `global_arena.offset = _frame_arena_save;` at the end of the loop body.

### Component 2: Snake Game Rewrite

#### [MODIFY] [snake.syl](file:///c:/Users/6041742/Downloads/SylDevelopmentandTesting/syl/examples/snake.syl)
- Completely rewrite to open an 800x600 window.
- Implement cell-based movement (grid size 20x20, meaning 40x30 grid).
- Track snake segments using two list variables `body_x` and `body_y`.
- To grow the snake, add new coordinates to the lists. To move without growing, shift elements left and decrement list counts by 1.
- Draw the snake as green rectangles, the food as a red circle, and render the live score at the top left.
- Add collision checks: wall collisions and self-collisions.
- Support game restart with `R` key, just like `web_game.syl`.

## Verification Plan

### Automated Tests
- Run `build_local.ps1` to compile the updated Syl compiler.
- Compile both games:
  - `cargo run --bin syl -- build examples/web_game.syl`
  - `cargo run --bin syl -- build examples/snake.syl`
- Verify successful compilation and generation of `web_game.exe` and `snake.exe`.

### Manual Verification
- Test `web_game.exe` to ensure no memory leak/scrambling happens and controls are responsive.
- Test `snake.exe` to ensure the game plays correctly in a separate graphical window, grows when eating food, increments the score, and handles keyboard controls.
