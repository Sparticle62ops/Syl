# API Reference: TerminalSnake

## Terminal UI / Graphics Lexicon Used
This script integrates raw Helix terminal graphics primitives:

- **Clear Terminal**: Clears the console window and resets cursor position.
- **Draw Terminal Box**: Renders a styled ANSI bounding box pane.
- **Move Cursor**: Sets cursor coordinates directly for grid redrawing.
- **Print Colored**: Prints styled ANSI color streams to stdout.
- **Sleep/Delay**: Low-latency thread sleep in milliseconds.
- **Wait for Key Press**: Non-blocking FFI keyboard reading mapped to `input_key`.

