# `three-d` Viewer Enhancement TODO

## Rendering Quality

A1. [x] Add ambient fill light + soft shadow mapping for the directional light.
A2. [ ] Load an HDR skybox for image-based lighting (IBL) on `PhysicalMaterial`.
A3. [ ] Implement MSAA via an intermediate multi-sampled render target.
A4. [ ] Add exposure/tone-mapping controls and HDR screenshot output.

## Model Display

A5. [ ] Exploded-view / ghost-mode toggle for inspecting IFC assemblies.
A6. [ ] Interactive section / cut-plane slider.
A7. [ ] Wireframe / edges overlay on top of shaded geometry.
A8. [ ] Back-face culling toggle.
A9. [ ] Orthographic camera mode alongside the current perspective view.

## Interaction

A10. [ ] Raycast hover / picking to highlight an element and show its metadata.
A11. [ ] Point-to-point measurement tool with on-canvas distance label.
A12. [ ] Auto-rotate / turntable animation mode.
A13. [ ] View-cube or preset view buttons (front, top, isometric).

## Data & Media

A14. [ ] GLB animation playback and skinning support.
A15. [ ] Extra model stats: surface area, bounding-box dimensions.
A16. [ ] Multi-model comparison / diff overlay with per-model tinting.
A17. [ ] 2D drawing / SVG export from the current camera view.
A18. [ ] Project thumbnail generation from the viewer for use in cards.

## App Integration

A19. [ ] IFC upload-to-GLB conversion pipeline status UI.
A20. [ ] Lightweight static preview on project cards instead of full 3D.
A21. [ ] Persist per-project viewer settings in local storage alongside `ViewState`.

## Library Migration / Manual Work Replacement

## High Value

B1. [x] Adopt `strum`/`strum_macros` for the Tag/Platform enums (`metadata/tags.rs`, `metadata/platforms.rs`) to remove the hand-written `label()`, `all()`, and `Display` match arms. Keep the per-variant `#[serde(rename)]` wire values (they are not snake_case-derivable: `3d_model`, `freecad`, `fusion_360`, `wip`) and the custom `color()` match; drive labels via `#[strum(serialize)]`.
B2. [x] Replace the hand-rolled KV rate limiter in `backend/src/utils.rs` (`RATE_LIMIT_WINDOW_SECONDS`/`RATE_LIMIT_MAX_REQUESTS` consts and the KV-based `check_rate_limit`) with Cloudflare's native Rate Limiting binding (`env.rate_limiter(...).limit(key)`), configured via `[[ratelimits]]` in `wrangler.toml`. Note: the `namespace_id` in `wrangler.toml` is a placeholder that must be set to a unique integer for the account before deploying.
B3. [x] Derive `Display` for `RequestError` in `frontend/src/data/error.rs` with `thiserror` instead of the manual `Display` impl. `message()` is kept as a one-line `self.to_string()` delegate because many call sites use it directly.

## Medium Value

B4. [ ] Remove brittle `html.replace(...)` class injection in `frontend/src/components/ui/markdown.rs` by inserting classes via `pulldown-cmark` events, or switch to `comrak` for custom renderer hooks.
B5. [ ] Apply `derive_more` (`Display`, `From`, `Constructor`, `AsRef`) to trim generic boilerplate across the workspace.

## Lower Priority

B6. [ ] Simplify `three_d_viewer/controls.rs` DOM event wiring by routing through Leptos `on:mouse:...` handlers instead of raw `web_sys` closures (`Closure`, `add_event_listener_with_callback`).
B7. [ ] Evaluate `openidconnect` in `backend/src/api/auth.rs` to replace the hand-built token exchange and Google profile fetch (GitHub endpoints stay manual).
B8. [ ] Keep the custom Markdown textarea editor (`markdown_editor.rs`) unless a mature Leptos/wasm editor crate appears; no good replacement exists yet.
