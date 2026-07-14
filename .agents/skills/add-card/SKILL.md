---
name: add-card
description: Add a new test card to cadiotheka/test_data/cards.json. Validates tags and platforms against the current enums, generates timestamps and placeholder counts, and ensures the fixture stays valid.
---

# Add a Card to the Cadiotheka Fixture

Use this skill when the user wants to add a new card entry to `test_data/cards.json`.

## Steps

1. Read the current fixture and enum definitions:
   - `test_data/cards.json`
   - `src/metadata/tags.rs`
   - `src/metadata/platforms.rs`

2. Ask the user for the card details if not provided:
   - title
   - author
   - description
   - tags (must be valid `Tag` variants)
   - supported_platforms (must be valid `Platform` variants)
   - downloads and favorites (default to 0 if not given)
   - timestamp (default to current UTC in RFC 3339 if not given)
   - icon_url (optional; omit or set to `null` if not given)

3. Validate each tag and platform name against the `serde(rename = ...)` values in the enums. If a name is invalid, suggest the closest valid alternative or ask the user for guidance.

4. Append the new card to the `cards` array in `test_data/cards.json`. Preserve formatting and keep the JSON valid.

5. Run `cargo test` and `cargo clippy --target wasm32-unknown-unknown` to ensure the fixture still deserializes correctly.

6. Summarize the added card and any validation issues found.

## Conventions

- Use RFC 3339 timestamps, e.g. `"2024-06-15T10:00:00Z"`.
- Keep downloads and favorites as non-negative integers.
- Vary titles, authors, descriptions, counts, and timestamps when adding multiple cards.
- Do not invent new tags or platforms; only use existing enum variants.
