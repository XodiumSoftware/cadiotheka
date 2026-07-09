---
name: add-component
description: Add a new reusable UI component to Cadiotheka under src/components/ following the existing Footer component pattern.
---

# Add a Cadiotheka Component

Use this skill when the user asks to add a new reusable UI component to the Cadiotheka application.

## Steps

1. Confirm the component name with the user if it is ambiguous or not provided.
2. Create `src/components/<name>.rs`.
3. Use the following template, adapted from `src/components/footer.rs`:

   ```rust
   //! <Human-readable description of the component>.

   /// State and rendering for the <name> component.
   #[derive(Default)]
   pub struct <Name> {
       // Add component-specific fields here.
   }

   impl <Name> {
       /// Draw the <name> component.
       pub fn show(&self, ui: &mut egui::Ui) {
           // Add egui rendering code here.
       }
   }
   ```

4. If the component needs new i18n strings, add them to `src/i18n.rs` following the existing pattern.
5. Register the component in `src/components/mod.rs` with `pub mod <name>;`. If no `mod.rs` exists yet, create it and expose `pub mod <name>;`.
6. Ensure `src/lib.rs` exposes the components module if it does not already: `pub mod components;`.
7. Run `cargo clippy` and address any warnings you introduced.

## Conventions

- Component structs use PascalCase and have a `Default` derive.
- Components expose a `pub fn show(&self, ui: &mut egui::Ui)` method unless mutable state is required.
- Keep components reusable and free of page-specific routing logic.
- Prefer `crate::i18n` for user-facing strings.
