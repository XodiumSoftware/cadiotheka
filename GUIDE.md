# Guide

## Table of Contents

- [Prerequisites](#prerequisites)
- [Build from Source](#build-from-source)
- [Run the Hub](#run-the-hub)
- [Run Tests](#run-tests)
- [Run Linting](#run-linting)
- [Build for Release](#build-for-release)
- [Troubleshooting](#troubleshooting)

---

## Prerequisites

- [Rust](https://www.rust-lang.org/) 1.85+ (edition 2024)

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/XodiumSoftware/cadiotheka.git
   cd cadiotheka
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. For an optimized release build:
   ```bash
   cargo build --release
   ```

## Run the Hub

Start the hub locally:

```bash
cargo run
```

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

## Build for Release

```bash
cargo build --release
```

The release binary will be located at:

```
target/release/cadiotheka
```

## Troubleshooting

### Build fails

- Verify Rust 1.85+ is installed and active:
  ```bash
  rustc --version
  cargo --version
  ```
- Make sure `rustup` is correctly configured:
  ```bash
  rustup show
  ```
- Try cleaning the build:
  ```bash
  cargo clean
  cargo build
  ```

### Clippy warnings

- Address all warnings; the project aims for a clean `cargo clippy` run.
- For forbidden unsafe code, do not use `unsafe` blocks. This is enforced via `[lints.rust]` in `Cargo.toml`.

---

<p align="right"><a href="#readme-top">▲</a></p>
