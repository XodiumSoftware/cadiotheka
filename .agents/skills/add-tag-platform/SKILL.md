---
name: add-tag-platform
description: Add a new Tag or Platform variant to Cadiotheka. Updates the enum, label, color, serde rename, all() list, tests, and fixture data where appropriate.
---

# Add a Tag or Platform to Cadiotheka

Use this skill when the user wants to add a new `Tag` or `Platform` variant.

## Steps

1. Identify which enum to extend:
   - `src/metadata/tags.rs` for content tags
   - `src/metadata/platforms.rs` for supported CAD platforms

2. Add the new variant to the enum with a `serde(rename = "snake_case_value")` attribute.

3. Add the user-facing `label()` match arm.

4. Add the Tailwind `color()` match arm. Pick a color that is distinct from existing ones and theme-appropriate.

5. Add the variant to the `all()` const array in stable order.

6. Update the `Display` impl if needed (it delegates to `label()`, so usually no change is required).

7. Add or update tests in the same file for the new label and serialization rename.

8. If the user wants to backfill the new tag/platform into existing fixture cards, update `test_data/cards.json` accordingly.

9. Run `cargo test` and `cargo clippy --target wasm32-unknown-unknown`.

10. Summarize the changes and any migration notes.

## Conventions

- Use descriptive `serde(rename)` values in `snake_case`.
- Tag colors should be `bg-*` Tailwind classes.
- Platform colors should be `text-*` Tailwind classes.
- Keep `all()` arrays stable; append new variants at the end unless there is a strong grouping reason.
