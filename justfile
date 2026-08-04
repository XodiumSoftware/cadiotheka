# Cadiotheka task runner
# Install `just` once: https://github.com/casey/just

_default:
    @just --list

# Lint the frontend for the WASM target (lints configured in workspace Cargo.toml)
lint-frontend:
    cd frontend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features

# Lint the frontend for the native target (lints configured in workspace Cargo.toml)
lint-frontend-native:
    cd frontend && cargo clippy --all-targets --all-features

# Lint the backend for the native target (lints configured in workspace Cargo.toml)
lint-backend:
    cd backend && cargo clippy --all-targets --all-features

# Lint the backend for the WASM target (lints configured in workspace Cargo.toml)
lint-backend-wasm:
    cd backend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features

# Run all lints (native + WASM for both crates)
lint: lint-frontend lint-frontend-native lint-backend lint-backend-wasm

# Check formatting across the workspace
fmt-check:
    cargo fmt --all -- --check

# Format the workspace
fmt:
    cargo fmt --all

# Run the workspace test suite
test:
    cargo test

# Run frontend tests only
test-frontend:
    cargo test -p frontend --lib

# Run backend tests only
test-backend:
    cargo test -p backend

# Find unused dependencies
machete:
    cargo machete

# Run the full validation suite used in CI
validate: lint fmt-check test machete

# Serve the backend locally with Wrangler
serve-backend:
    cd backend && npx wrangler dev

# Serve the frontend locally (backend must be running on port 8787)
serve-frontend:
    cd frontend && trunk serve --port 8080

# Build the frontend for release
build-frontend:
    cd frontend && trunk build --release

# Build the backend worker bundle for Wrangler
build-backend:
    cd backend && worker-build
