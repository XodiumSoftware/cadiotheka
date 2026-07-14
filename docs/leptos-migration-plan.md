# Cadiotheka → Leptos Migration Plan

This plan moves Cadiotheka from the egui/eframe immediate-mode stack to a
Leptos + Trunk web-app stack, matching the patterns already proven in
[xodium.org](https://github.com/XodiumSoftware/xodium.org).

## Goals

1. Reuse existing Xodium build tooling and conventions (`Trunk.toml`,
   `index.html`, `Cargo.toml` workspace layout).
2. Replace egui layout fighting with normal HTML/CSS (Flexbox/Grid).
3. Keep all domain logic (search, cards, tags, platforms) in Rust.
4. Preserve the current feature set: header, hub grid, search modal,
   project popup, keyboard shortcuts, dotted background, Xodium orange theme.
5. Keep the project license (AGPL-3.0) and `unsafe_code = "forbid"`.

## Current vs. Target Architecture

| Layer | Current (egui) | Target (Leptos) |
|-------|----------------|-----------------|
| Entry | `src/main.rs` + `eframe::WebRunner` | `src/main.rs` mounts `<App/>` to the DOM |
| App shell | `eframe::App::ui` | `src/app.rs` Leptos component with router |
| Pages | `src/pages/hub.rs` structs | `src/pages/hub.rs` Leptos page component |
| Components | egui widgets in `src/components/` | Leptos components in `src/components/` |
| Styling | `src/theme.rs` + egui visuals | Tailwind + CSS variables in `styles/` |
| Icons | Phosphor icon font | SVG icon components or Phosphor web font |
| Search modal | `egui::Modal` | HTML `<dialog>` or custom modal component |
| Keyboard shortcuts | `Keycap` widget + `ui.input` | `wasm_bindgen` keydown listener + `<kbd>` elements |
| Cards data | embedded JSON fixture | same fixture, loaded at build or via resource |
| i18n | `src/i18n.rs` string constants | same constants, or Leptos `i18n` crate later |

## Files to Keep, Rewrite, or Delete

### Keep (mostly unchanged)

- `Cargo.toml` — update deps, keep profiles and lints.
- `Trunk.toml` — already compatible; may need `tailwind` build step.
- `index.html` — replace canvas shell with Leptos mount point.
- `assets/` — favicon, manifest, service worker.
- `test_data/cards.json` — continue as fixture.
- `src/i18n.rs` — constants can still feed labels.
- `src/tags.rs` and `src/platforms.rs` — pure enums, no change.
- `src/utils.rs` — helper functions mostly unchanged.
- Domain engines under `src/engines/`:
  - `filter.rs`, `query.rs`, `suggestions.rs` — pure logic, keep as-is.
  - Card data types may move to a shared `src/data.rs`.

### Rewrite (egui → Leptos + HTML)

- `src/main.rs` — native entry point becomes browser-only Leptos mount.
- `src/lib.rs` — export `App` component instead of `CadiothekaApp`.
- `src/app.rs` — root component with global signals (`search_open`, `query`).
- `src/pages/hub.rs` — Leptos page: loading, error, grid, modal wiring.
- `src/components/header.rs` — `<header>` with nav button + search trigger.
- `src/components/footer.rs` — `<footer>` static component.
- `src/components/card.rs` — card component with click handlers.
- `src/components/grid.rs` — grid layout component, empty state.
- `src/components/search_bar.rs` — input + dropdown + keyboard navigation.
- `src/components/project_popup.rs` — HTML dialog/modal.
- `src/components/builders/dotted_background.rs` — CSS/SVG background.
- `src/components/builders/keycap.rs` — replace with `<Kbd>` component.
- `src/theme.rs` — convert to CSS variables / Tailwind config.

### Delete

- `eframe`, `egui`, `egui_extras`, `egui_phosphor_icons` dependencies.
- Native `eframe::run_native` code path.
- `src/main.rs` desktop block.

## Dependency Changes

Remove:

```toml
# current
egui = "0.35.0"
eframe = { version = "0.35.0", default-features = false, features = ["glow"] }
egui_extras = { version = "0.35.0", features = ["image"] }
egui_phosphor_icons = "0.4.0"
```

Add (Leptos CSR with Trunk):

```toml
[dependencies]
leptos = { version = "0.7", features = ["csr"] }
leptos_router = { version = "0.7", features = ["csr"] }
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
web-sys = { version = "0.3", features = [
    "Window", "Document", "Element", "HtmlElement",
    "KeyboardEvent", "EventTarget", "HtmlDialogElement"
] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
console_error_panic_hook = "0.1"

[build-dependencies]
# if using Tailwind via build.rs, keep build.rs or switch to Trunk hooks
```

Tailwind setup (optional but recommended, matching xodium.org style):

- Add `tailwindcss` build in Trunk or `build.rs`.
- Create `styles/tailwind.css` with `@tailwind` directives.
- Add Xodium orange palette in `tailwind.config.js` or CSS variables.

## Phase-by-Phase Plan

### Phase 0: Prep (1–2 days)

1. Create a feature branch: `git checkout -b leptos-migration`.
2. Add Leptos dependencies and remove egui dependencies.
3. Update `Trunk.toml` / `index.html` for Leptos CSR mount point:
   ```html
   <body>
     <div id="app"></div>
     <script type="module">
       import init from './cadiotheka.js';
       init();
     </script>
   </body>
   ```
4. Add Tailwind build step if xodium.org uses it.
5. Verify `trunk serve` compiles a minimal `App` component.

### Phase 1: Foundation (2–3 days)

1. Rewrite `src/main.rs` to mount the app:
   ```rust
   use leptos::*;
   use cadiotheka::App;

   fn main() {
       console_error_panic_hook::set_once();
       mount_to_body(App);
   }
   ```
2. Rewrite `src/lib.rs` to export `App`.
3. Rewrite `src/app.rs` as root component with global signals:
   - `search_open: RwSignal<bool>`
   - `query: RwSignal<String>`
   - `view: RwSignal<View>`
4. Create `src/theme.css` or `styles/main.css` with Xodium orange CSS variables.
5. Port `src/components/footer.rs` and `src/components/header.rs` to simple HTML
   components first.

### Phase 2: Hub Page (3–4 days)

1. Port `src/pages/hub.rs`:
   - Load cards once with `create_local_resource` or `spawn_local`.
   - Render loading, error, and grid states.
   - Wire global `query` signal into `SearchEngine`.
2. Port `src/components/card.rs` and `src/components/grid.rs`:
   - HTML card markup.
   - CSS Grid for responsive columns.
   - Empty-state button + `<Kbd>` shortcut hint.
3. Port `src/components/project_popup.rs` as HTML `<dialog>`.

### Phase 3: Search UX (3–4 days)

1. Port `src/components/search_bar.rs`:
   - Controlled `<input>` bound to `query` signal.
   - Render grouped suggestions as HTML list.
   - Keyboard navigation with `window` keydown listener.
2. Port `src/components/builders/keycap.rs` to a small `<Kbd>` component.
3. Build a command-palette modal (HTML dialog) matching current design.
4. Re-implement shortcuts:
   - `Ctrl + S` — open search.
   - `Alt + H` — switch to Hub.
   - `Ctrl + C` — clear search.
   - `Esc` / `Backspace` on empty input — close modal / clear.
   - `Enter` / arrow keys — navigate suggestions.

### Phase 4: Polish (2–3 days)

1. Port dotted background to CSS/SVG.
2. Port Xodium orange theme to Tailwind classes / CSS variables.
3. Ensure icons (magnifying glass, platform icons) render with SVG or Phosphor
   web font.
4. Verify responsive behavior.
5. Update `AGENTS.md` and README with new stack and commands.

### Phase 5: CI & Cleanup (1–2 days)

1. Update `.github/workflows/` to use `cargo-leptos` or keep Trunk.
2. Remove obsolete egui files and dead code.
3. Run `cargo clippy` and fix all warnings.
4. Add tests for pure Rust modules (`engines`, `tags`, `platforms`).
5. Manual QA of all keyboard shortcuts and the search modal.

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| Leptos 0.7 API churn | Pin versions; follow official Leptos book examples. |
| Native desktop support lost | Decide if needed; Leptos is web-only. If desktop is required later, consider Tauri. |
| Async card loading | Use `create_local_resource` or `spawn_local` with `wasm-bindgen-futures`. |
| Keyboard shortcut conflicts | Add `prevent_default` only when the modal is open. |
| Accessibility | Use semantic HTML (`<dialog>`, `<button>`, `<nav>`, `<main>`) from the start. |
| Tailwind build complexity | Start with plain CSS variables, add Tailwind once the app compiles. |

## Recommended First Commit

A minimal "Hello World" Leptos app that compiles with Trunk and renders a single
styled button. This validates the build pipeline before any UI logic is ported.

## Open Questions

1. Should Cadiotheka remain a pure client-side app, or do we want Leptos SSR?
   (xodium.org appears to use Trunk CSR; CSR is simpler and matches.)
2. Do we keep the embedded card fixture or fetch `cards.json` at runtime?
3. Which icon system does xodium.org use? (Phosphor web font, Lucide, SVG?)
4. Should we use `leptos_i18n` now or keep the simple `i18n.rs` constant approach?

## Suggested Decision

Proceed with **Leptos CSR + Trunk**, plain CSS variables first, Tailwind second.
It aligns with xodium.org, removes the egui layout friction you've already hit,
and gives a better foundation for future UI work.
