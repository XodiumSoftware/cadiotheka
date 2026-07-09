# Cadiotheka — Agents Context

## Project at a Glance

- **Name:** Cadiotheka
- **Type:** Rust hub application (browser-only WebAssembly via `eframe`)
- **Language:** Rust (edition 2024)
- **Build Tool:** Cargo + [Trunk](https://trunkrs.dev/)
- **Target:** `wasm32-unknown-unknown`
- **License:** AGPL-3.0

## APIs & Tools

| Category            | Technology                              | Purpose                            |
|---------------------|-----------------------------------------|------------------------------------|
| **Core Language**   | [Rust](https://www.rust-lang.org/) latest stable | Systems/application language       |
| **UI Framework**    | [egui](https://github.com/emilk/egui) / [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) | Browser GUI                        |
| **Web Bundler**     | [Trunk](https://trunkrs.dev/)           | WASM build and dev server          |
| **Build Tool**      | [Cargo](https://doc.rust-lang.org/cargo/) | Build automation                   |
| **CI/CD**           | GitHub Actions                          | Builds, tests, releases            |

## Quick Commands

```bash
# Lint the project
cargo clippy

# Run the test suite
cargo test

# Serve the web app locally
trunk serve --port 8080

# Build for the web (WASM)
trunk build

# Build for release
trunk build --release
```

## Project Structure

```
Cadiotheka/
├── Cargo.toml          # Rust package configuration
├── Trunk.toml          # Trunk web bundler configuration
├── index.html          # Trunk page shell (root)
├── src/                # Source directory
│   ├── main.rs         # Native entry point and web Trunk entry point
│   ├── lib.rs          # Public export of CadiothekaApp
│   ├── app.rs          # CadiothekaApp state and UI
│   └── i18n.rs         # Centralized user-facing strings
├── assets/             # Static web assets
│   ├── favicon.svg
│   ├── manifest.json
│   └── sw.js           # Service worker for offline/PWA support
├── .github/            # GitHub Actions workflows
└── docs/               # Documentation
```

## Architecture

### Entry Points

1. **`src/main.rs`** — Web entry point. Uses `eframe::WebRunner` when compiled for `wasm32` via Trunk.
2. **`src/lib.rs`** — Public re-export of [`CadiothekaApp`].
3. **`src/app.rs`** — `CadiothekaApp` struct and [`eframe::App`] UI implementation.
4. **`src/i18n.rs`** — Centralized user-facing strings.

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
2. Add `mod registry;` to `src/lib.rs` if needed.
3. Keep public APIs minimal and well-named.
4. Add tests for new behavior.

### Adding Utilities

1. Add small reusable helpers to an existing module or a new `src/utils.rs`.
2. Prefer pure functions and avoid global mutable state.

## Memory System

This project uses Claude Code's persistent memory in `.claude/memory/`. These files persist across sessions and different PCs. Review `MEMORY.md` for existing context about the user and project.
