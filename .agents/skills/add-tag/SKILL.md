---
name: add-tag
description: Add a new content Tag to Cadiotheka by adding an enum variant to frontend/src/metadata/tags.rs. No database changes are needed.
---

# Add a Tag to Cadiotheka

Tags are hardcoded as the `Tag` enum in `frontend/src/metadata/tags.rs`. Project
rows still store tag wire ids as JSON arrays, so the frontend resolves labels
and colors through the enum.

## Steps

1. Open `frontend/src/metadata/tags.rs`.

2. Add a new variant to the `Tag` enum. Use `snake_case` for the variant name;
   for ids that start with a digit or contain special characters, add a
   `#[serde(rename = "...")]` attribute:

   ```rust
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, Serialize, Deserialize)]
   #[serde(rename_all = "snake_case")]
   pub enum Tag {
       ...
       /// Description of the new tag.
       NewTag,
   }
   ```

3. Update the three match arms in `impl Tag`:
   - `id()` returns the stable wire id (`"new_tag"`).
   - `label()` returns the user-facing label (`"New Tag"`).
   - `color()` returns a Tailwind `bg-*`/`text-*` inline style string.

4. Run `cargo fmt --all`.

5. Run `cargo clippy --all-targets --all-features` for the backend and
   `cd frontend && cargo clippy --target wasm32-unknown-unknown --all-targets --all-features`.

6. Run `cargo test`.

7. Summarize the change. Existing project rows with the new wire id will render
   correctly once the enum recognizes it; old ids that are removed from the
   enum will no longer resolve to a label or color.

## Conventions

- Wire ids are stable, lowercase, and use `snake_case`.
- Labels are user-facing and may differ from the id (e.g. `WIP` vs `wip`).
- Color strings are Tailwind-style inline CSS, usually a `background-color` for
  badge rendering.
