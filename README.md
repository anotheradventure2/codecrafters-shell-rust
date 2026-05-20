[![progress-banner](https://backend.codecrafters.io/progress/shell/77edbf33-1214-4887-8567-c2bf0eea367c)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is a Rust solution to the
["Build Your Own Shell" Challenge](https://app.codecrafters.io/courses/shell/overview).

## Project Structure

```
src/
├── lib.rs       # Crate root — declares all modules
├── main.rs      # Binary entry point — owns the REPL loop
├── path.rs      # PATH scanning and executable lookup
└── builtins.rs  # Shell builtin commands (type, echo, pwd, exit)
```

### Why `lib.rs` + `main.rs`?

This project uses the **library crate** pattern:

- **`lib.rs`** is the crate root. It declares `pub mod path;` and `pub mod builtins;`.
  Because `lib.rs` is the root, every module can import every other module via `crate::`.
- **`main.rs`** is a thin binary that imports the library (`use codecrafters_shell::...`)
  and runs the REPL loop.

This is the idiomatic Rust way to structure a project with multiple modules — it lets
modules cross-import each other and makes testing easier.

### Module Responsibilities

| Module | Purpose |
|--------|---------|
| `path.rs` | Scans `$PATH` directories for executable files. Provides `list_executables()` (builds the full list), `find_executable()` (looks up a command by name), and `get_executables()` (scans a single directory). |
| `builtins.rs` | Handles shell builtin commands. `handle_type()` resolves whether a command is a builtin or external executable. `execute_command()` spawns external processes. |
| `main.rs` | The REPL loop: prints `$`, reads input, splits into `[command, args]`, and dispatches via `match`. |
| `lib.rs` | Crate root. Just declares the modules as public. |

### How Cross-Imports Work

Because `lib.rs` is the crate root, `crate::` refers to the library:

```rust
// builtins.rs can import path.rs
use crate::path;

// main.rs imports the library by crate name
use codecrafters_shell::builtins;
use codecrafters_shell::path;
```

### Key Design Decisions

- **PATH is built once** at startup (`list_executables()`), not on every command.
- **`find_executable` uses a lifetime** (`'a`) to return a borrowed `&PathBuf` — no unnecessary cloning.
- **`get_executables` handles missing directories gracefully** — `.into_iter().flatten()` skips non-existent directories instead of panicking.
- **`splitn(2, ' ')`** splits input into at most 2 parts: the command and the rest of the arguments.

## Running

```sh
# Run the shell locally
cargo run

# Clippy
cargo clippy

# Submit to CodeCrafters
codecrafters submit
```
