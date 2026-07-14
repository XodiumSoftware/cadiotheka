---
name: add-component
description: Scaffold a new Leptos component in Cadiotheka following project conventions, including imports, i18n usage, and styling patterns.
---

# Add a Leptos Component to Cadiotheka

Use this skill when the user wants to create a new UI component.

## Steps

1. Determine the component's purpose and where it belongs:
   - `src/components/sections/` for page-level sections (header, footer, projects)
   - `src/components/cards/` for card widgets
   - `src/components/ui/` for reusable primitives
   - `src/components/effects/` for visual effects

2. Create the file under the appropriate directory.

3. Write the component using the project's conventions:
   - Use `#[component]` from `leptos::prelude::*`.
   - Accept props with `#[prop(into)]`, `#[prop(optional)]`, or `#[prop(default = ...)]` as needed.
   - Use `t!` / `t_string!` and `use_i18n()` for any user-facing text.
   - Prefer Tailwind utility classes. Use DaisyUI classes (`btn`, `card`, `badge`, etc.) where appropriate.
   - Keep accessibility in mind: labels, `role`, `aria-*`, and keyboard handling.

4. If the component should be publicly usable, export it from `src/lib.rs` under the `components` module.

5. Add a small test or doctest if the component has pure logic worth testing. Browser-only rendering is hard to unit-test; don't force a test for pure view code.

6. Run `cargo clippy --target wasm32-unknown-unknown` and `cargo test`.

7. Summarize the new component, its props, and where it is exported.

## Conventions

- Component functions return `impl IntoView`.
- Avoid hard-coded English strings; externalize via `locales/en.json`.
- Use `CornerFrame`, `OverflowRow`, and other existing UI primitives when appropriate.
- Keep components focused and small.
