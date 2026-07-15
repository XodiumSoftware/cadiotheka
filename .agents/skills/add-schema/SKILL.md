---
name: add-schema
description: Add a new D1 SQL schema file for the cadiotheka-backend, name it consistently, and update the deployment docs in README.md and GUIDE.md.
---

# Add a new backend schema

Use this skill when the user wants to add a new D1 database schema file under `cadiotheka-backend/schemas/`.

## Steps

1. Ask the user for the schema name (e.g. `projects`, `likes`, `comments`) if it isn't already clear from their request.
2. Create the new SQL file at `cadiotheka-backend/schemas/<name>.sql`.
3. Write clean SQLite/D1-compatible SQL. Prefer:
   - `IF NOT EXISTS` on table creation.
   - `TEXT` for UUID/id fields.
   - `INTEGER` for booleans/flags with a `DEFAULT` value.
   - Foreign keys that reference existing tables (currently `accounts(id)`).
   - Inline comments only where the intent is non-obvious.
4. Update `README.md` and `GUIDE.md` under the backend deployment/seed sections so the example `npx wrangler d1 execute` commands reference the new schema file (or show multiple files being applied in order).
5. Do **not** modify existing schema files unless the user explicitly asks for a migration or alter statement.
6. Do **not** run wrangler or touch `wrangler.toml`.

## Naming convention

- `snake_case` names for schema files.
- Keep files small and focused: one logical entity per file (e.g. `accounts.sql`, `projects.sql`).

## Example output

After adding a `projects` schema, `GUIDE.md` should show:

```bash
npx wrangler d1 execute cadiotheka-db --file=schemas/accounts.sql
npx wrangler d1 execute cadiotheka-db --file=schemas/projects.sql
```
