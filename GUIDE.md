# Guide

## Table of Contents

- [Prerequisites](#prerequisites)
- [Project Layout](#project-layout)
- [Build the Frontend](#build-the-frontend)
- [Run Frontend Locally](#run-frontend-locally)
- [Run Backend Locally](#run-backend-locally)
- [Run Tests](#run-tests)
- [Run Linting](#run-linting)
- [Build for Release](#build-for-release)
- [Deploy the Backend](#deploy-the-backend)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

- [Rust](https://www.rust-lang.org/) — latest stable toolchain (edition 2024)
- `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/)

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
rustup target add wasm32-unknown-unknown
```

Install Trunk:

```bash
cargo install --locked trunk
```

Verify your toolchain:

```bash
rustc --version
cargo --version
trunk --version
```

## Project Layout

This repository is a Cargo workspace with two members:

- `cadiotheka-frontend/` — Leptos CSR browser app compiled to `wasm32-unknown-unknown`.
- `cadiotheka-backend/` — Cloudflare Pages Functions Rust backend using D1.

Most day-to-day development commands are run from inside one of those crates.

## Build the Frontend

1. Clone the repository:
   ```bash
   git clone https://github.com/XodiumSoftware/cadiotheka.git
   cd cadiotheka/cadiotheka-frontend
   ```

2. Build and bundle the web app with Trunk:
   ```bash
   trunk build
   ```

3. For an optimized release build:
   ```bash
   trunk build --release
   ```

The output is placed in `cadiotheka-frontend/dist/`.

## Run Frontend Locally

The frontend needs the backend dev server running on port `8787`. Trunk will
proxy `/api/*` requests there automatically.

Start the backend first:

```bash
cd cadiotheka-backend
npx wrangler dev
```

In a second terminal, start Trunk:

```bash
cd cadiotheka-frontend
trunk serve --port 8080
```

Then open <http://localhost:8080/index.html#dev> in a browser. The `#dev` hash
disables the service worker cache so you always see the latest build.

Trunk rebuilds automatically when you edit the project.

## Run Backend Locally

The backend is a Cloudflare Pages Functions Rust worker. First build the WASM bundle, then run Wrangler:

```bash
cd cadiotheka-backend
cargo install worker-build --version 0.7.5 --force
worker-build
npx wrangler dev
```

The local API is available at <http://localhost:8787/api/accounts> by default.

## Seed the Database

Apply the schema and seed the local D1 database with fixture accounts:

```bash
cd cadiotheka-backend
npx wrangler d1 execute cadiotheka-db --file=schemas/accounts.sql --local
npx wrangler d1 execute cadiotheka-db --file=scripts/seed_accounts.sql --local
```

To seed a production database, omit the `--local` flag after updating `wrangler.toml` with the real `database_id`.

## Build the Backend

Build the WASM bundle that Wrangler serves:

```bash
cd cadiotheka-backend
worker-build
```

The generated worker shim is placed in `cadiotheka-backend/build/`.

## Build for Release

```bash
cd cadiotheka-frontend
trunk build --release
```

The release site is placed in `cadiotheka-frontend/dist/`. Upload that folder to
your static host (e.g. Cloudflare Pages alongside the backend).

## Run Tests

Run the full workspace test suite:

```bash
cargo test
```

To run only the frontend tests:

```bash
cargo test -p cadiotheka-frontend
```

To run only the backend tests:

```bash
cargo test -p cadiotheka-backend
```

## Run Linting

Lint the frontend with the WASM target:

```bash
cd cadiotheka-frontend
cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
```

Lint the backend:

```bash
cd cadiotheka-backend
cargo clippy --all-targets --all-features -- -D warnings
```

To also run tests and checks together:

```bash
cargo test && cargo clippy --target wasm32-unknown-unknown -- -D warnings
```

## Troubleshooting

### Build fails

- Verify the latest stable Rust toolchain is installed and active:
  ```bash
  rustc --version
  cargo --version
  rustup show
  ```
- Ensure the `wasm32-unknown-unknown` target is installed:
  ```bash
  rustup target list --installed
  ```
- Try cleaning the build:
  ```bash
  cargo clean
  trunk build
  ```

### Clippy warnings

- Address all warnings; the project aims for a clean `cargo clippy` run.
- For forbidden unsafe code, do not use `unsafe` blocks. This is enforced via `[lints.rust]` in each crate's `Cargo.toml`.

### Backend local dev issues

- Ensure `npx wrangler dev` is run from `cadiotheka-backend/`.
- The D1 database ID in `wrangler.toml` must match the database you created for production; local dev uses a local D1 binding automatically.

## Deploy the Backend

1. Create a D1 database:
   ```bash
   cd cadiotheka-backend
   npx wrangler d1 create cadiotheka-db
   ```

2. Update `wrangler.toml` with the database ID from step 1.

3. Apply the schema:
   ```bash
   npx wrangler d1 execute cadiotheka-db --file=schemas/accounts.sql
   ```

4. Build and deploy:
   ```bash
   npx wrangler deploy
   ```

---

<p align="right"><a href="#readme-top">▲</a></p>
