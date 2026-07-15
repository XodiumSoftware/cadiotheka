---
name: add-card
description: Add a new fixture project to cadiotheka-backend/scripts/seed_projects.sql. Validates tags and platforms against the current enums, generates timestamps and placeholder counts, and keeps the seed script valid.
---

# Add a Project to the Cadiotheka Seed Data

Use this skill when the user wants to add a new fixture project to the backend D1 seed script.

## Steps

1. Read the current seed script and enum definitions:
   - `cadiotheka-backend/scripts/seed_projects.sql`
   - `cadiotheka-frontend/src/metadata/tags.rs`
   - `cadiotheka-frontend/src/metadata/platforms.rs`

2. Ask the user for the project details if not provided:
   - `title`
   - `author` (display name; must match an existing account's username/display_name)
   - `author_id` (UUID of the account that owns the project)
   - `description` (short summary shown on the card)
   - `extended_desc` (markdown description shown in the project detail modal; default to the short `description` if not given)
   - `tags` (must be valid `Tag` variants)
   - `supported_platforms` (must be valid `Platform` variants)
   - `downloads` and `favorites` (default to 0 if not given)
   - `timestamp` (default to current UTC in RFC 3339 if not given)
   - `icon_url` (optional; default `NULL`)

3. Generate a UUID v4 for `id`. Do not reuse an existing id.

4. Validate each tag and platform name against the `serde(rename = ...)` values in the enums. If a name is invalid, suggest the closest valid alternative or ask the user for guidance.

5. Append a new `INSERT` row to `cadiotheka-backend/scripts/seed_projects.sql`.
   - The last existing row ends with `);` if it is the final row, or `),` if more rows follow.
   - Change the previous final row's terminator from `);` to `),` before appending.
   - Add the new row as the final row terminated with `);`.
   - Store `tags` and `supported_platforms` as JSON arrays, e.g. `["3d_model","vehicle"]`.
   - Preserve the existing formatting and keep the SQL valid.

6. Run validation:
   ```bash
   cd cadiotheka-backend && cargo clippy --all-targets --all-features -- -D warnings
   cd cadiotheka-frontend && cargo test --lib
   ```

7. If the local D1 database is already set up, optionally re-seed it:
   ```bash
   cd cadiotheka-backend
   npx wrangler d1 execute cadiotheka-db --file=scripts/seed_projects.sql --local
   ```

8. Summarize the added project and any validation issues found.

## Conventions

- Use RFC 3339 timestamps, e.g. `"2024-06-15T10:00:00Z"`.
- Keep downloads and favorites as non-negative integers.
- Vary titles, authors, descriptions, counts, and timestamps when adding multiple projects.
- Do not invent new tags or platforms; only use existing enum variants.
