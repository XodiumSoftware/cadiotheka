---
name: add-builder
description: Add a new builder-style component to Cadiotheka under src/components/builders/ following the existing DottedBackground pattern.
---

# Add a Builder Component

Use this skill when the user wants to create a new configurable builder widget in `src/components/builders/`, following the pattern established by `DottedBackground`.

## Pattern

A builder component is a small, self-contained egui widget that:

1. Lives in its own file under `src/components/builders/`.
2. Uses a struct with private fields and a consuming builder API (`fn builder() -> Self`, `fn field(self, value) -> Self`).
3. Has a `Default` impl that provides sensible defaults.
4. Exposes a `pub fn build(self, ui: &mut egui::Ui)` method that performs the actual rendering.

## Steps

1. Read `src/components/builders/dotted_background.rs` as the canonical example.
2. Create `src/components/builders/<name>.rs` matching the file-level doc comment, struct, `Default`, builder methods, and `build` pattern.
3. Add the new module to `src/lib.rs` inside the `pub mod components { pub mod builders { ... } }` block.
4. Re-export the new type alongside `DottedBackground` in `src/lib.rs` so it is available as `crate::components::NewBuilder`.
5. Keep the component pure (no side effects) and theme-aware by reading from `ui.visuals()` where possible.
6. Do not change unrelated components.

## Naming

- File name: `snake_case.rs`
- Struct name: `PascalCase`
- Builder methods: use the field name as the method name (e.g., `fn spacing(self, spacing: f32) -> Self`)
