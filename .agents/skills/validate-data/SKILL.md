---
name: validate-data
description: Validate all Cadiotheka test fixtures (test_data/cards.json and test_data/accounts.json) against the current Rust data types.
---

# Validate Cadiotheka Data Fixtures

Use this skill when the user wants to verify the fixture files or after editing `test_data/cards.json` or `test_data/accounts.json`.

## Steps

1. Read the current fixture files:
   - `test_data/cards.json`
   - `test_data/accounts.json`

2. Read the current data types and enums:
   - `src/data/card.rs`
   - `src/data/account.rs`
   - `src/metadata/tags.rs`
   - `src/metadata/platforms.rs`

3. Validate every card entry:
   - `tags` must contain only known `Tag` variant names (the `#[serde(rename = "...")]` values).
   - `supported_platforms` must contain only known `Platform` variant names.
   - `timestamp` must be a valid RFC 3339 string.
   - `downloads` and `favorites` must be non-negative integers.
   - `icon_url` must be a string URL or `null`.

4. Validate every account entry:
   - `id` must be a valid UUID.
   - `username` must be unique across the fixture.
   - `email` must contain `@` and a plausible domain.
   - `role` must be `creator` or `admin`.
   - `created_at` must be a valid RFC 3339 string.
   - `verified` must be a boolean.

5. If the user wants to fix errors, replace unknown tags/platforms with the closest valid alternatives or ask for guidance when no close match exists.

6. Run `cargo test` to confirm both fixtures deserialize correctly through the Rust types.

7. Report the outcome concisely:
   - List any invalid entries, unknown tags/platforms, duplicates, or malformed fields.
   - If everything passes, state that clearly.

## Validation Shortcuts

You can run a quick fixture check with:

```bash
cargo test data::
```

A deserialization panic in `data::card::load_cards` or `data::account::load_accounts` means the JSON is stale or contains unknown variants.

## Conventions

- Use the *serialized* enum names from `#[serde(rename = "...")]`, not the Rust variant names.
- Keep fixture entries self-contained and deterministic.
- Do not invent new tags, platforms, or roles unless the user asks you to extend the enums too.
