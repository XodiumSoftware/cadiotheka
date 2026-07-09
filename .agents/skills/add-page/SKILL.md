---
name: add-page
description: Add a new page module to Cadiotheka under src/pages/ following the existing Hub page pattern.
---

# Add a Cadiotheka Page

Use this skill when the user asks to add a new page to the Cadiotheka application.

## Steps

1. Confirm the page name with the user if it is ambiguous or not provided.
2. Create `src/pages/<name>.rs`.
3. Use the following template, adapted from `src/pages/hub.rs`:

   ```rust
   //! <Human-readable description of the page>.

   use crate::i18n;

   /// State for the <name> page UI.
   #[derive(Default)]
   pub struct <Name> {
       // Add page-specific fields here.
   }

   impl <Name> {
       /// Renders the <name> page UI.
       pub fn show(&mut self, ui: &mut egui::Ui) {
           ui.heading(i18n::<Name>::HEADING);
           // Add page-specific widgets here.
       }
   }
   ```

4. If the page needs new i18n strings, add them to `src/i18n.rs` in a new module-style constants block (follow the existing `Hub` pattern).
5. Register the page in `src/lib.rs` with `pub mod pages;` if a `pages` module does not already exist, or add `pub mod <name>;` inside `src/pages/mod.rs`.
6. If the app should navigate to or instantiate the new page, wire it into `src/app.rs` following the existing page routing pattern.
7. Run `cargo clippy` and address any warnings you introduced.

## Conventions

- Page structs use PascalCase and have a `Default` derive.
- Pages expose a `pub fn show(&mut self, ui: &mut egui::Ui)` method.
- Keep page modules focused on a single screen.
- Prefer `crate::i18n` for user-facing strings.
