---
name: validate-data
description: Validate the Cadiotheka backend seed scripts against the current Rust data types and enum definitions.
---

# Validate Cadiotheka Seed Data

Use this skill when the user wants to verify the fixture data or after editing `cadiotheka-backend/scripts/seed_accounts.sql` or `cadiotheka-backend/scripts/seed_projects.sql`.

## Steps

1. Read the current seed scripts:
   - `cadiotheka-backend/scripts/seed_accounts.sql`
   - `cadiotheka-backend/scripts/seed_projects.sql`

2. Read the current data types and enums:
   - `cadiotheka-frontend/src/data/card.rs`
   - `cadiotheka-frontend/src/data/account.rs`
   - `cadiotheka-frontend/src/metadata/tags.rs`
   - `cadiotheka-frontend/src/metadata/platforms.rs`
   - `cadiotheka-backend/src/api/accounts.rs`
   - `cadiotheka-backend/src/api/projects.rs`

3. Validate every account row:
   - `id` must be a valid UUID.
   - `username` must be unique across the seed.
   - `email` must contain `@` and a plausible domain.
   - `role` must be `creator` or `admin`.
   - `created_at` must be a valid RFC 3339 string.
   - `verified` must be `0` or `1`.

4. Validate every project row:
   - `id` must be a valid UUID.
   - `author_id` must reference an existing account `id`.
   - `tags` must be a JSON array containing only known `Tag` variant names (the `#[serde(rename = "...")]` values).
   - `supported_platforms` must be a JSON array containing only known `Platform` variant names.
   - `timestamp` must be a valid RFC 3339 string.
   - `downloads` and `favorites` must be non-negative integers.
   - `icon_url` must be a string URL or `NULL`.

5. If the user wants to fix errors, replace unknown tags/platforms with the closest valid alternatives or ask for guidance when no close match exists.

6. Run validation:
   ```bash
   cd cadiotheka-backend && cargo clippy --all-targets --all-features -- -D warnings
   cd cadiotheka-frontend && cargo test --lib
   ```

7. Report the outcome concisely:
   - List any invalid entries, unknown tags/platforms, duplicates, malformed fields, or dangling `author_id` references.
   - If everything passes, state that clearly.

## Validation Shortcuts

You can run a quick check with:

```bash
cargo test -p cadiotheka-frontend data::
```

A deserialization panic in `data::card::tests` or `data::account::tests` means a seed value is stale or contains unknown variants.

## Conventions

- Use the *serialized* enum names from `#[serde(rename = "...")]`, not the Rust variant names.
- Keep seed entries self-contained and deterministic.
- Do not invent new tags, platforms, or roles unless the user asks you to extend the enums too.
