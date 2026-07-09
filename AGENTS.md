# Cadiotheka — Agents Context

## Project at a Glance

- **Name:** Cadiotheka
- **Type:** Rust binary / hub application
- **Language:** Rust (edition 2024)
- **Build Tool:** Cargo
- **License:** AGPL-3.0

## APIs & Tools

| Category            | Technology                              | Purpose                            |
|---------------------|-----------------------------------------|------------------------------------|
| **Core Language**   | [Rust](https://www.rust-lang.org/) 1.85+| Systems/application language       |
| **Build Tool**      | [Cargo](https://doc.rust-lang.org/cargo/) | Build automation                   |
| **CI/CD**           | GitHub Actions                          | Builds, tests, releases            |

## Quick Commands

```bash
# Build the project
cargo build

# Run the hub locally
cargo run

# Run the test suite
cargo test

# Run linting
cargo clippy

# Build for release
cargo build --release
```

## Project Structure

```
Cadiotheka/
├── Cargo.toml          # Rust package configuration
├── src/                # Source directory
│   └── main.rs         # Application entry point
├── .github/            # GitHub Actions workflows
└── docs/               # Documentation
```

## Architecture

### Entry Points

1. **`src/main.rs`** — Application entry point. Starts the Cadiotheka hub.

## Key Conventions

- Follow Rust naming conventions and idioms.
- Keep code safe: `unsafe_code` is forbidden via `[lints.rust]` in `Cargo.toml`.
- Address all `cargo clippy` warnings.
- Use clear module boundaries as the project grows.
- Prefer immutable data and explicit error handling (`Result`, `Option`).

## Testing

- Use `cargo test` to run the test suite.
- Add tests for new modules and edge cases where applicable.

## Important Notes

- This is a hub for CAD creators, not a programming library.
- The project is in early development; structure will evolve.

## CI/CD

GitHub Actions workflows in `.github/workflows/` handle building, testing, and releases.

## Adding Components

### Adding a New Module

1. Create a new file under `src/` (e.g., `src/registry.rs`).
2. Add `mod registry;` to `src/main.rs` if needed.
3. Keep public APIs minimal and well-named.
4. Add tests for new behavior.

### Adding Utilities

1. Add small reusable helpers to an existing module or a new `src/utils.rs`.
2. Prefer pure functions and avoid global mutable state.

## Memory System

This project uses Claude Code's persistent memory in `.claude/memory/`. These files persist across sessions and different PCs. Review `MEMORY.md` for existing context about the user and project.
