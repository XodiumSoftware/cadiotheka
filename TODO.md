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

- [ ] **Make tags/platforms clickable.** The card renders them as small buttons but they do nothing. Clicking a tag should append `#tag` to the search query; clicking a platform should append `#platform` or its label.
- [ ] **Filter matches prefix only.** `#blend` matches `Blender`, but `#model` won't match `3D Model` because the label starts with `3D`. Consider tokenizing labels or substring matching.
- [ ] **Author suggestion rendering.** The popup shows `@author:AuthorName` for author suggestions, but the suggestion text itself is just the name. That looks correct.
- [ ] **Sort suggestions.** They always appear first. Consider only showing sort suggestions when the active prefix is `@`.

## Data & Assets

- [ ] **Cards fixture is hardcoded.** `hub.rs` embeds `test_data/cards.json` via `include_str!`. Document that this is temporary and plan a real data source (JSON fetch, GitHub repo index, etc.).
- [ ] **Missing icon support.** `icon_url` exists but no code loads or displays it. Add image loading via `egui_extras` or lazy URL-based icons.
- [ ] **Validate fixture against enums.** There is a `validate-cards` skill. Use it to keep `cards.json` in sync with `Tag` and `Platform`.

## Tooling & CI

- [ ] **Add a `cargo fmt` check.** No `rustfmt.toml` is visible; standard formatting is fine, but CI should enforce it.
- [x] **Release profile.** Measured `opt-level = "z"` + `lto = true` + `strip = true` against the previous `opt-level = 2` profile. The aggressive profile shrank the release WASM from **8.8 MB** to **7.9 MB** (~10% smaller) at the cost of a longer build (~1m 51s → ~3m 00s). Updated `[profile.release]` accordingly.
- [ ] **Trunk caching.** CI likely rebuilds everything each run. Add `Swatinem/rust-cache` if not already present.

## Documentation

- [x] **README mismatch.** Replaced the misleading `docs.rs/cadiotheka` badge with a GitHub license badge that points to the actual repository.
- [x] **CONTRIBUTING setup.** Updated `CONTRIBUTING.md` with `cargo clippy` and `trunk serve --port 8080` instructions.
