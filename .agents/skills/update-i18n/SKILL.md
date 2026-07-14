---
name: update-i18n
description: Add or update a localized string in Cadiotheka. Ensures the key is added to locales/en.json and used via t! or t_string! in components.
---

# Update i18n Strings in Cadiotheka

Use this skill when the user wants to add or change user-facing text.

## Steps

1. Identify the string and where it is used (or will be used) in `src/`.

2. Add the key to `locales/en.json` under the appropriate namespace:
   - Top-level keys for global strings (e.g. `skip_to_content`).
   - `header.*` for header text.
   - `projects.*` for project grid text.
   - `search.*` for search modal text.
   - `footer.*` for footer text.

3. Use the key in the component:
   - For dynamic/view content: `t!(i18n, namespace.key)`
   - For string props: `t_string!(i18n, namespace.key)`

4. Remove any hard-coded string literals that the new key replaces.

5. Run `cargo test` and `cargo clippy --target wasm32-unknown-unknown`. leptos_i18n is strongly typed, so missing keys will fail at compile time.

6. Summarize the added/updated keys.

## Conventions

- Use `snake_case` keys.
- Keep keys namespaced by feature area.
- Avoid concatenating translated fragments; prefer complete phrases when possible.
- Use `t_string!` for props that require `&'static str` or plain `String`.
