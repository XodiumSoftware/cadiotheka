---
name: add-schema
description: Add a new D1 SQL schema file for the backend, name it consistently, and update the deployment docs in README.md and GUIDE.md.
---

# Add a new backend schema

Use this skill when the user wants to add a new D1 database schema file under `backend/schemas/`.

## Steps

1. Ask the user for the schema name (e.g. `projects`, `likes`, `comments`) if it isn't already clear from their request.
2. Determine whether this is a **new table** or a **change to an existing table**.
   - **New table:** create the SQL file at `backend/schemas/<name>.sql`.
   - **Change to an existing table:** create a numbered migration file at `backend/migrations/NNNN_<description>.sql` using `ALTER TABLE` statements (see existing migrations for examples). Do **not** edit the original `backend/schemas/<name>.sql` for existing deployments.
3. Write clean SQLite/D1-compatible SQL. Prefer:
   - `IF NOT EXISTS` on table creation.
   - `TEXT` for UUID/id fields.
   - `INTEGER` for booleans/flags with a `DEFAULT` value.
   - Foreign keys that reference existing tables (currently `accounts(id)`).
   - Inline comments only where the intent is non-obvious.
4. Update `README.md` and `GUIDE.md` under the backend deployment/seed sections so the example `npx wrangler d1 execute` commands reference the new schema or migration file (or show multiple files being applied in order).
5. Do **not** modify existing schema files unless the user explicitly asks for a migration or alter statement.
6. Do **not** run wrangler or touch `wrangler.toml`.

## Naming convention

- `snake_case` names for schema files.
- Keep files small and focused: one logical entity per file (e.g. `accounts.sql`, `projects.sql`).
- Migration files are numbered sequentially in `backend/migrations/` and describe the change, e.g. `0001_add_project_versions_file_size.sql`.

## Example output

After adding a `projects` schema, `GUIDE.md` should show:

```bash
npx wrangler d1 execute cadiotheka-db --file=schemas/accounts.sql
npx wrangler d1 execute cadiotheka-db --file=schemas/projects.sql
```
