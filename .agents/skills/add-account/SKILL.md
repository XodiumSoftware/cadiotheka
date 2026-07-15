---
name: add-account
description: Add a new fixture account to cadiotheka-backend/scripts/seed_accounts.sql. Generates a UUID, validates roles, timestamps, and email shape, and keeps the seed script valid.
---

# Add an Account to the Cadiotheka Seed Data

Use this skill when the user wants to add a new fixture account to the backend D1 seed script.

## Steps

1. Read the current seed script:
   - `cadiotheka-backend/scripts/seed_accounts.sql`

2. Ask the user for account details if not provided:
   - `username` (unique, required)
   - `display_name` (defaults to username if not given)
   - `email` (must contain `@`, required)
   - `role` (must be `creator` or `admin`; defaults to `creator`)
   - `bio` (optional; default empty string)
   - `avatar_url` (optional; default `NULL`)
   - `created_at` (default to current UTC in RFC 3339)
   - `verified` (default `0`, use `1` for verified)
   - `project_ids` (optional list of UUIDs; stored only in the frontend model, not in D1)

3. Generate a UUID v4 for `id`. Do not reuse an existing id or username.

4. Append a new `INSERT` row to `cadiotheka-backend/scripts/seed_accounts.sql`.
   - The last existing row ends with `);` if it is the final row, or `),` if more rows follow.
   - Change the previous final row's terminator from `);` to `),` before appending.
   - Add the new row as the final row terminated with `);`.
   - Preserve the existing formatting and keep the SQL valid.

5. Run backend clippy and frontend tests to make sure nothing is broken:
   ```bash
   cd cadiotheka-backend && cargo clippy --all-targets --all-features -- -D warnings
   cd cadiotheka-frontend && cargo test --lib
   ```

6. If the local D1 database is already set up, optionally re-seed it:
   ```bash
   cd cadiotheka-backend
   npx wrangler d1 execute cadiotheka-db --file=scripts/seed_accounts.sql --local
   ```

7. Summarize the added account and any validation issues found.

## Conventions

- Use RFC 3339 timestamps, e.g. `"2024-06-15T10:00:00Z"`.
- `role` must be one of `creator` or `admin`.
- `verified` is an SQLite integer: `0` for false, `1` for true.
- `avatar_url` is `NULL` when absent.
- Keep seed data deterministic unless the user asks for realistic data.
