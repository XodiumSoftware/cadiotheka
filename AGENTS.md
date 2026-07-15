# Cadiotheka — Agents Context

## Project at a Glance

- **Name:** Cadiotheka
- **Type:** Rust workspace with two members:
  - `cadiotheka-frontend` — browser-only WebAssembly Leptos CSR app.
  - `cadiotheka-backend` — Cloudflare Pages Functions Rust worker with D1 database.
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
cd cadiotheka-frontend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings

# Lint the backend
cd cadiotheka-backend && cargo clippy --all-targets --all-features -- -D warnings

# Run the test suite
cargo test

# Serve the web app locally (backend must be running on port 8787)
cd cadiotheka-backend && npx wrangler dev
cd cadiotheka-frontend && trunk serve --port 8080

# Build for the web (WASM)
cd cadiotheka-frontend && trunk build

# Build for release
cd cadiotheka-frontend && trunk build --release

# Build the backend WASM bundle for Wrangler
cd cadiotheka-backend && worker-build
```

## Testing

- Use `cargo test` to run the full workspace test suite.
- Run frontend tests with `cargo test -p cadiotheka-frontend --lib`.
- Run backend tests with `cargo test -p cadiotheka-backend`.
- Add tests for new modules and edge cases where applicable.

## Data Sources

- The backend owns the canonical data in D1.
- Fixture data lives as SQL seed scripts in `cadiotheka-backend/scripts/` (e.g. `seed_accounts.sql`, `seed_projects.sql`).
- The frontend does **not** embed JSON fixtures; it fetches accounts from `/api/accounts` and projects from `/api/projects`.
- Schemas live in `cadiotheka-backend/schemas/` (one entity per file, e.g. `accounts.sql`, `projects.sql`).

## Project Structure

```
cadiotheka/
├── Cargo.toml                         # Workspace configuration
├── cadiotheka-frontend/
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
│       ├── metadata/                  # Tag and platform enums
│       └── utils.rs
├── cadiotheka-backend/
│   ├── Cargo.toml
│   ├── wrangler.toml                  # D1 binding, worker entry
│   ├── schemas/                       # D1 schema files
│   ├── scripts/                       # Seed SQL scripts
│   └── src/
│       ├── lib.rs                     # Router, DB_BINDING constant
│       └── api/                       # Route handlers (accounts.rs, projects.rs, ...)
├── .github/workflows/                 # CI/CD
└── docs/                              # Documentation
```

## Architecture

### Entry Points

1. **`cadiotheka-frontend/src/main.rs`** — Web entry point. Uses `leptos::mount_to_body` when compiled for `wasm32` via Trunk.
2. **`cadiotheka-frontend/src/lib.rs`** — Public re-export of the `App` component and explicit module registration.
3. **`cadiotheka-frontend/src/app.rs`** — `App` state and [`leptos::IntoView`] UI implementation.
4. **`cadiotheka-backend/src/lib.rs`** — Cloudflare Worker entry point with `#[event(fetch)]` and route definitions.
5. **`cadiotheka-backend/src/api/*.rs`** — Route handlers grouped by entity.

## Key Conventions

- Follow Rust naming conventions and idioms.
- Keep code safe: `unsafe_code` is forbidden via `[lints.rust]` in `Cargo.toml`.
- Address all `cargo clippy` warnings.
- Use clear module boundaries as the project grows.
- Prefer immutable data and explicit error handling (`Result`, `Option`).
- **Register modules and re-exports in `src/lib.rs` explicitly; do not use `mod.rs` files.**
- Use `snake_case` for all Rust source filenames. Compound module names should be split with underscores (e.g. `project_card.rs`, `search_modal.rs`, `corner_frame.rs`, `project_list.rs`), not concatenated.
- When adding crate dependencies, look up the latest version on [crates.io](https://crates.io) rather than guessing or reusing an old version from another crate in the workspace.
- Backend route handlers live under `cadiotheka-backend/src/api/` and are wired in `cadiotheka-backend/src/lib.rs`.
- Backend `DB_BINDING` is a single `pub(crate) const` in `cadiotheka-backend/src/lib.rs` reused by API modules.
- Tags and platforms are stored as JSON arrays in D1 and deserialize into the frontend enums via `serde(rename)`.
- `verified` columns are stored as SQLite integers (`0`/`1`), not booleans, because D1 returns them as numbers.

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
