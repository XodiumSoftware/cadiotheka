# Cadiotheka — Agents Context

## Project at a Glance

- **Name:** Cadiotheka
- **Type:** Rust workspace with two members:
  - `frontend` — browser-only WebAssembly Leptos CSR app.
  - `backend` — Cloudflare Pages Functions Rust worker with D1 database.
- **Language:** Rust (edition 2024)
- **Build Tool:** Cargo + [Trunk](https://trunkrs.dev/)
- **Target:** `wasm32-unknown-unknown`
- **License:** AGPL-3.0

## APIs & Tools

| Category            | Technology                              | Purpose                            |
|---------------------|-----------------------------------------|------------------------------------|
| **Core Language**   | [Rust](https://www.rust-lang.org/) latest stable | Systems/application language       |
| **UI Framework**    | [leptos](https://github.com/leptos-rs/leptos) | Browser GUI                        |
| **Web Bundler**     | [Trunk](https://trunkrs.dev/)           | WASM build and dev server          |
| **Build Tool**      | [Cargo](https://doc.rust-lang.org/cargo/) | Build automation                   |
| **CI/CD**           | GitHub Actions                          | Builds, tests, releases            |

## Quick Commands

```bash
# Lint the frontend
# Pedantic lints are configured in the workspace Cargo.toml and denied by default.
cd frontend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features

# Lint the backend
cd backend && cargo clippy --all-targets --all-features

# Run the test suite
cargo test

# Serve the web app locally (backend must be running on port 8787)
cd backend && npx wrangler dev
cd frontend && trunk serve --port 8080

# Build for the web (WASM)
cd frontend && trunk build

# Build for release
cd frontend && trunk build --release

# Build the backend WASM bundle for Wrangler
cd backend && worker-build
```

## Testing

- Use `cargo test` to run the full workspace test suite.
- Run frontend tests with `cargo test -p frontend --lib`.
- Run backend tests with `cargo test -p backend`.
- Add tests for new modules and edge cases where applicable.

## Data Sources

- The backend owns the canonical data in D1.
- The frontend does **not** embed JSON fixtures; it fetches accounts from `/data/accounts` and projects from `/data/projects`.
- Schemas live in `backend/schemas/` (one entity per file, e.g. `accounts.sql`, `projects.sql`).

## Project Structure

```
cadiotheka/
├── Cargo.toml                         # Workspace configuration
├── frontend/
│   ├── Cargo.toml
│   ├── Trunk.toml                     # Trunk dev server + API proxy
│   ├── index.html
│   └── src/                           # Leptos app source
│       ├── main.rs
│       ├── lib.rs                     # Explicit module registration, no mod.rs files
│       ├── app.rs
│       ├── data/                      # AccountData, CardData, fetch functions
│       ├── contexts/                  # Leptos reactive contexts (no mod.rs)
│       ├── components/                # UI components
│       ├── engines/                   # Search/suggestion logic
│       ├── metadata/                  # Tag enums
│       └── utils.rs
├── backend/
│   ├── Cargo.toml
│   ├── wrangler.toml                  # D1 binding, worker entry
│   ├── schemas/                       # D1 schema files (new tables)
│   ├── migrations/                    # Numbered ALTER TABLE migrations for existing deployments
│   ├── scripts/                       # Seed SQL scripts
│   └── src/
│       ├── lib.rs                     # Router, DB_BINDING constant
│       └── api/                       # Route handlers (accounts.rs, projects.rs, ...)
├── .github/workflows/                 # CI/CD
└── docs/                              # Documentation
```

## Architecture

### Entry Points

1. **`frontend/src/main.rs`** — Web entry point. Uses `leptos::mount_to_body` when compiled for `wasm32` via Trunk.
2. **`frontend/src/lib.rs`** — Public re-export of the `App` component and explicit module registration.
3. **`frontend/src/app.rs`** — `App` state and [`leptos::IntoView`] UI implementation.
4. **`backend/src/lib.rs`** — Cloudflare Worker entry point with `#[event(fetch)]` and route definitions.
5. **`backend/src/api/*.rs`** — Route handlers grouped by entity.

## Key Conventions

- Follow Rust naming conventions and idioms.
- Keep code safe: `unsafe_code` is forbidden via `[lints.rust]` in `Cargo.toml`.
- Address all `cargo clippy` warnings.
- Use clear module boundaries as the project grows.
- Prefer immutable data and explicit error handling (`Result`, `Option`).
- **Register modules and re-exports in `src/lib.rs` explicitly.** Do not use `mod.rs` files. Modules may be grouped in `src/lib.rs` using nested `mod` blocks (e.g. `components { pub mod cards; }`) to mirror the directory structure, but do not nest `mod` declarations inside any other `.rs` file. Every crate-level module must be declared in `src/lib.rs` for `frontend` and `backend`.
- Use `snake_case` for all Rust source filenames. Compound module names should be split with underscores (e.g. `project_card.rs`, `search_modal.rs`, `corner_frame.rs`, `project_list.rs`), not concatenated.
- When adding crate dependencies, look up the latest version on [crates.io](https://crates.io) rather than guessing or reusing an old version from another crate in the workspace.
- Backend route handlers live under `backend/src/api/` and are wired in `backend/src/lib.rs`.
- Backend `DB_BINDING` is a single `pub(crate) const` in `backend/src/lib.rs` reused by API modules.
- Tags are hardcoded as an enum in `frontend/src/metadata/tags.rs`. Project rows store tag wire ids as JSON arrays; the frontend resolves labels and colors through the enum.
- `verified` columns are stored as SQLite integers (`0`/`1`), not booleans, because D1 returns them as numbers.
- **Do not add `//` inline comments.** Use `///` doc comments (or `//!` module docs) to explain intent; keep the code itself self-documenting.

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
2. Add `mod registry;` (or `pub mod registry;`) to `src/lib.rs` if needed, inside an existing top-level group or as a new top-level module.
3. Keep public APIs minimal and well-named.
4. Add tests for new behavior.

### Adding Utilities

1. Add small reusable helpers to an existing module or a new `src/utils.rs`.
2. Prefer pure functions and avoid global mutable state.

## Memory System

This project uses Claude Code's persistent memory in `.claude/memory/`. These files persist across sessions and different PCs. Review `MEMORY.md` for existing context about the user and project.
