# Cadiotheka task runner
# Install `just` once: https://github.com/casey/just

# PowerShell is the default shell on Windows (the plain `sh` there rejects `&&`).
[windows]
set shell := ["powershell.exe", "-NoProfile", "-Command"]

# The frontend's public directory; `dist/` is only created by a release build.
_pages-dir := "frontend/dist"

_default:
    @just --list

# Lint the frontend for the WASM target (lints configured in workspace Cargo.toml)
[unix]
lint-frontend:
    cd frontend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

[windows]
lint-frontend:
    Set-Location frontend; cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

# Lint the frontend for the native target (lints configured in workspace Cargo.toml)
[unix]
lint-frontend-native:
    cd frontend && cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

[windows]
lint-frontend-native:
    Set-Location frontend; cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

# Lint the backend for the native target (lints configured in workspace Cargo.toml)
[unix]
lint-backend:
    cd backend && cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

[windows]
lint-backend:
    Set-Location backend; cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

# Lint the backend for the WASM target (lints configured in workspace Cargo.toml)
[unix]
lint-backend-wasm:
    cd backend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

[windows]
lint-backend-wasm:
    Set-Location backend; cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -W clippy::pedantic -D warnings -A clippy::must_use_candidate -A clippy::too_many_lines

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
[unix]
serve-backend:
    cd backend && npx wrangler dev

[windows]
serve-backend:
    Set-Location backend; npx wrangler dev

# Serve the frontend locally (backend must be running on port 8787)
[unix]
serve-frontend:
    cd frontend && trunk serve --port 8080

[windows]
serve-frontend:
    Set-Location frontend; trunk serve --port 8080

# Build the frontend for release
[unix]
build-frontend:
    cd frontend && trunk build --release

[windows]
build-frontend:
    Set-Location frontend; trunk build --release

# Build the backend worker bundle for Wrangler
[unix]
build-backend:
    cd backend && worker-build

[windows]
build-backend:
    Set-Location backend; worker-build

# Build both targets for a manual Cloudflare Pages deploy
build: build-frontend build-backend
    @echo "Deploy with: npx wrangler pages deploy {{ _pages-dir }} --project-name=cadiotheka"

# Remove frontend build output and cached tooling
[unix]
clean-frontend:
    rm -rf {{ _pages-dir }}
    rm -rf frontend/.trunk

[windows]
clean-frontend:
    Remove-Item -Recurse -Force {{ _pages-dir }}, frontend/.trunk -ErrorAction SilentlyContinue; exit 0

# Remove the backend worker bundle and local Wrangler state
[unix]
clean-backend:
    rm -rf backend/build
    rm -rf backend/.wrangler

[windows]
clean-backend:
    Remove-Item -Recurse -Force backend/build, backend/.wrangler -ErrorAction SilentlyContinue; exit 0

# Remove every build artifact, including Cargo's cache
clean: clean-frontend clean-backend
    cargo clean
