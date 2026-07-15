---
name: add-account
description: Add a new test account to cadiotheka/test_data/accounts.json. Generates a UUID, validates roles, timestamps, and email shape, and ensures the fixture stays valid.
---

# Add an Account to the Cadiotheka Fixture

Use this skill when the user wants to add a new account entry to `test_data/accounts.json`.

## Steps

1. Read the current fixture:
   - `test_data/accounts.json`

2. Ask the user for account details if not provided:
   - `username` (unique, required)
   - `display_name` (defaults to username if not given)
   - `email` (must contain `@`, required)
   - `role` (must be `creator` or `admin`; defaults to `creator`)
   - `bio` (optional; default empty string)
   - `avatar_url` (optional; default `null`)
   - `created_at` (default to current UTC in RFC 3339)
   - `verified` (default `false`)

3. Generate a UUID v4 for `id`. Do not reuse an existing id or username.

4. Append the new account to the `accounts` array in `test_data/accounts.json`. Preserve formatting and keep the JSON valid.

5. Run `cargo test` to ensure the fixture still deserializes correctly.

6. Summarize the added account and any validation issues found.

## Conventions

- Use RFC 3339 timestamps, e.g. `"2024-06-15T10:00:00Z"`.
- `role` must serialize as `"creator"` or `"admin"` (snake_case).
- Keep fixture accounts self-contained and deterministic unless the user asks for realistic data.
- Do not add fields beyond those defined in `src/data/account.rs`.
