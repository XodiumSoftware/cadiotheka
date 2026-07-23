# Cadiotheka task runner
# Install `just` once: https://github.com/casey/just

_default:
    @just --list

# Lint the frontend for the WASM target with pedantic lints
lint-frontend:
    cd cadiotheka-frontend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -W clippy::pedantic -D warnings

# Lint the frontend for the native target with pedantic lints
lint-frontend-native:
    cd cadiotheka-frontend && cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings

# Lint the backend for the native target with pedantic lints
lint-backend:
    cd cadiotheka-backend && cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings

# Lint the backend for the WASM target with pedantic lints
lint-backend-wasm:
    cd cadiotheka-backend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -W clippy::pedantic -D warnings

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
    cargo test -p cadiotheka-frontend --lib

# Run backend tests only
test-backend:
    cargo test -p cadiotheka-backend

# Find unused dependencies
machete:
    cargo machete

# Run the full validation suite used in CI
validate: lint fmt-check test machete

# Serve the backend locally with Wrangler
serve-backend:
    cd cadiotheka-backend && npx wrangler dev

# Serve the frontend locally (backend must be running on port 8787)
serve-frontend:
    cd cadiotheka-frontend && trunk serve --port 8080

# Build the frontend for release
build-frontend:
    cd cadiotheka-frontend && trunk build --release

# Build the backend worker bundle for Wrangler
build-backend:
    cd cadiotheka-backend && worker-build
