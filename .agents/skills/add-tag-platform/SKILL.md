---
name: add-tag-platform
description: Add a new Tag or Platform variant to Cadiotheka. Updates the enum variant (serde rename + strum label), the color match arm, tests, and fixture data where appropriate.
---

# Add a Tag or Platform to Cadiotheka

Use this skill when the user wants to add a new `Tag` or `Platform` variant.

## Steps

1. Identify which enum to extend:
   - `src/metadata/tags.rs` for content tags
   - `src/metadata/platforms.rs` for supported CAD platforms

2. Add the new variant to the enum with two attributes:
   - `#[serde(rename = "snake_case_value")]` for the wire format (must match what the backend stores).
   - `#[strum(serialize = "User-facing label")]` for the display label (used by `label()`, `Display`, and suggestions).

3. Add the Tailwind `color()` match arm. Pick a color that is distinct from existing ones and theme-appropriate.

4. No `all()` list or `Display` impl is needed: `iter()` comes from the `EnumIter` derive and `Display` is derived from the `#[strum(serialize)]` labels. If a test asserts the total variant count (e.g. `Tag::iter().count()`), bump it.

5. Add or update tests in the same file for the new label and serialization rename.

6. If the user wants to backfill the new tag/platform into existing fixture cards, update `test_data/cards.json` accordingly.

7. Run `cargo test` and `cargo clippy --target wasm32-unknown-unknown`.

8. Summarize the changes and any migration notes.

## Conventions

- Use descriptive `serde(rename)` values in `snake_case`; these are the persisted wire values.
- The `#[strum(serialize = "...")]` value is the user-facing label and may differ from the wire value (e.g. `wip` vs `"WIP"`, `3d_model` vs `"3D Model"`).
- Tag colors should be `bg-*` Tailwind classes.
- Platform colors should be `text-*` Tailwind classes.
- Keep enum variant order stable; append new variants at the end unless there is a strong grouping reason.
