---
name: run-validation
description: Run the full Cadiotheka validation suite (cargo test, clippy, fmt, and cargo-machete) and report the results clearly.
---

# Run the Cadiotheka Validation Suite

Use this skill when the user wants to verify the project state or before finishing a change.

## Steps

1. Run the unit tests on the native host target:
   ```bash
   cargo test
   ```

2. Run clippy on the WASM target with warnings as errors:
   ```bash
   cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
   ```

3. Check formatting:
   ```bash
   cargo fmt --all -- --check
   ```

4. Run `cargo machete` to detect unused dependencies:
   ```bash
   cargo machete
   ```
   If `cargo-machete` is not installed, install it first:
   ```bash
   cargo binstall cargo-machete --no-confirm --force
   ```

5. Report the outcome concisely:
   - List any failing commands and relevant error lines.
   - If everything passes, state that clearly.
   - Do not claim validation passed unless you actually ran the commands and saw success.

## Notes

- `cargo test` runs on the native target because the project no longer forces a global `[build] target`.
- Clippy must target `wasm32-unknown-unknown` for WASM-equivalent checks.
- `cargo machete` is a required check; if it reports unused dependencies, fix or ignore them before claiming validation passed.
- If a command fails, stop and report the error; do not hide it.
