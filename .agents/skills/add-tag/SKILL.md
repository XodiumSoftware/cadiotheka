---
name: add-tag
description: Add a new content Tag to Cadiotheka. Tags are stored in the D1 database, so this means adding a row to the seed schema (schemas/tags.sql); the API serves it automatically and the frontend resolves labels through MetadataContext.
---

# Add a Tag to Cadiotheka

Tags are database records served over `GET /data/tags`. The frontend resolves
them through `MetadataContext`, so adding one is a schema/seed change only.

## Steps

1. Add a row to `backend/schemas/tags.sql`.

2. Use the existing `INSERT OR IGNORE INTO ... VALUES (...)` block. Each row is
   `(id, label, color)`:
   - `id`: the stable wire value stored on project rows, in `snake_case`
     (e.g. `3d_model`, `wip`). It must be unique.
   - `label`: the user-facing label (e.g. `3D Model`, `WIP`).
   - `color`: a Tailwind `bg-*` class. Pick one distinct from existing rows.

3. Re-apply the schema to refresh local data:
   ```bash
   cd backend
   npx wrangler d1 execute cadiotheka --file=schemas/tags.sql --local
   ```

   Existing rows are preserved because inserts use `OR IGNORE`.

4. No frontend changes are needed for a single new tag. If you are building the
   admin CRUD (TODO C1b), prefer the admin UI so the change is applied to the
   database at runtime rather than via a migration.

5. Run `cargo test` and `cargo clippy` for both crates.

6. Summarize the change and any migration notes (e.g. re-apply the schema in
   production).

## Conventions

- `id` values are stable wire values in `snake_case`; `label` is user-facing and
  may differ from the id (e.g. `wip` vs `WIP`, `3d_model` vs `3D Model`).
- Tag colors should be `bg-*` Tailwind classes.
