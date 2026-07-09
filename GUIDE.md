# Guide

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build for the Web](#build-for-the-web)
- [Run Locally](#run-locally)
- [Run Tests](#run-tests)
- [Run Linting](#run-linting)
- [Build for Release](#build-for-release)
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

## Build for the Web

1. Clone the repository:
   ```bash
   git clone https://github.com/XodiumSoftware/cadiotheka.git
   cd cadiotheka
   ```

2. Build and bundle the web app with Trunk:
   ```bash
   trunk build
   ```

3. For an optimized release build:
   ```bash
   trunk build --release
   ```

The output is placed in `dist/`.

## Run Locally

Start a development server with Trunk:

```bash
trunk serve --port 8080
```

Then open <http://localhost:8080/index.html#dev> in a browser. The `#dev` hash
disables the service worker cache so you always see the latest build.

Trunk rebuilds automatically when you edit the project.

## Build for Release

```bash
trunk build --release
```

The release site is placed in `dist/`. Upload that folder to any static host
such as GitHub Pages.

## Run Tests

```bash
cargo test
```

## Run Linting

```bash
cargo clippy
```

To also run tests and checks together:

```bash
cargo test && cargo clippy
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
- For forbidden unsafe code, do not use `unsafe` blocks. This is enforced via `[lints.rust]` in `Cargo.toml`.

---

<p align="right"><a href="#readme-top">▲</a></p>
