# Cadiotheka Improvement TODO

## Code Quality & Architecture

- [x] **Add tests.** Added tests for `filter.rs`, `suggestions.rs`, `utils.rs`, `card.rs`, and `grid.rs`; fixed pre-existing broken tests in `search_bar.rs` and `query.rs`. Tests compile for `wasm32-unknown-unknown`; run with `wasm-pack test` in a browser environment.
- [ ] **Hide implementation details.** `lib.rs` re-exports many internals. Consider keeping engine internals private (`parse_query`, `from_cards`) and exposing only `SearchEngine` and `Suggestion`.
- [ ] **Reduce clones in search.** `filter.rs` clones every matching card. If `CardData` stays owned, consider returning indices or `Rc<CardData>` for large catalogs.
- [ ] **Use `&str` over owned strings.** `ParsedQuery` fields like `filter`, `filters`, and `author` allocate on every keystroke. For short-lived query parsing this is fine, but benchmarking may show it matters.

## UI/UX

- [ ] **Empty state.** `Grid.show` renders nothing when `cards` is empty. Show a friendly "No results" message with a clear-search action.
- [ ] **Loading state.** `Hub::default()` parses JSON synchronously at startup. For a real fixture this is fine, but document the path or add async loading later.
- [ ] **Header view is confusing.** `Header.show` writes `self.view` every frame, so `CadiothekaApp` copies it back into `self.view`. This works but is redundant; let `Header` expose the value or make `Hub` own view selection.
- [ ] **Footer always visible.** Bottom panel plus central panel means the central area can be cramped on small screens. Consider a collapsible footer or moving links to an about page.
- [ ] **Icon placeholders.** They are deterministic by title length, so two cards starting with different letters can get the same color. Hash the title or first letter for better differentiation.

## Search & Filtering

- [x] **Make tags/platforms clickable.** Tag and platform buttons on cards now trigger a `CardAction::Filter`. The action bubbles up through `Grid` to `Hub`, which appends `#TagName` or `#PlatformName` to the search query.
- [x] **Filter matches prefix only.** Tag/platform filtering now tokenizes labels on whitespace and non-alphanumeric characters, then performs case-insensitive substring matching per token. `#model` now matches `3D Model` and `#fusion` matches `Fusion 360` while preserving prefix matches like `#blend` → `Blender`.
- [x] **Author suggestion rendering.** Author suggestions store just the author name and render as `@author:AuthorName` in the popup, inserting the same prefixed form into the query. Added tests in `search_bar.rs` to lock in this behavior.
- [x] **Sort suggestions.** Sort directives no longer appear first by default. `SearchEngine::suggestions(query)` only includes sort suggestions when the active token starts with `@`, so the initial popup shows titles, authors, tags, and platforms instead.

## Data & Assets

- [x] **Cards fixture is hardcoded.** `src/fixture.rs` embeds `test_data/cards.json` via `include_str!` and documents that this is temporary. The planned future source is a remote JSON endpoint or generated index (e.g., GitHub repo index) loaded at runtime.
- [ ] **Missing icon support.** `icon_url` exists but no code loads or displays it. Add image loading via `egui_extras` or lazy URL-based icons.
- [x] **Validate fixture against enums.** `src/fixture.rs` loads `test_data/cards.json` into `Tag` and `Platform` enums. `cargo test` now validates the fixture on every run, and a clear error is returned if a tag or platform drifts out of sync.

## Tooling & CI

- [x] **Release profile.** Measured `opt-level = "z"` + `lto = true` + `strip = true` against the previous `opt-level = 2` profile. The aggressive profile shrank the release WASM from **8.8 MB** to **7.9 MB** (~10% smaller) at the cost of a longer build (~1m 51s → ~3m 00s). Updated `[profile.release]` accordingly.

## Documentation

- [x] **README mismatch.** Replaced the misleading `docs.rs/cadiotheka` badge with a GitHub license badge that points to the actual repository.
- [x] **CONTRIBUTING setup.** Updated `CONTRIBUTING.md` with `cargo clippy` and `trunk serve --port 8080` instructions.
