---
name: validate-cards
description: Validate and extend cadiotheka/test_data/cards.json against the current Tag and Platform enums.
---

# Validate and Extend Card Fixtures

Use this skill when adding or editing test cards in `test_data/cards.json`.

## Steps

1. Read `test_data/cards.json`.
2. Read `src/tags.rs` to collect the current valid `Tag` variants (look for `#[serde(rename = "...")]` attributes).
3. Read `src/platforms.rs` to collect the current valid `Platform` variants (look for `#[serde(rename = "...")]` attributes).
4. Validate every card entry:
   - `tags` must contain only known tag names.
   - `supported_platforms` must contain only known platform names.
   - `timestamp` must be a valid RFC 3339 string.
   - `downloads` and `favorites` must be non-negative integers.
5. If the user wants to add new test cases, generate additional entries using only valid tag and platform names. Vary titles, authors, descriptions, counts, and timestamps.
6. If the user wants to fix errors, replace unknown tags/platforms with the closest valid alternatives or ask the user for guidance if no close match exists.
7. Run `cargo clippy` (WASM target is fine) to confirm the JSON still deserializes correctly.

## Tag/Platform Name Rules

- Use the *serialized* names from the `#[serde(rename = "...")]` attributes, not the Rust enum variant names.
- Common tags include: `3d_model`, `2d_drawing`, `parametric`, `fabrication`, `robotics`, `furniture`, `vehicle`, `architecture`, `electronics`, `tooling`, `lighting`, `diy`, `interior`, `engineering`, `aerospace`, `decor`, `medical`, `game_asset`, `art`, `educational`, `wip`.
- Common platforms include: `blender`, `freecad`, `sketchup`, `fusion_360`, `kicad`, `autocad`, `solidworks`, `onshape`, `tinkercad`, `step`, `mesh`.

## Validation Shortcuts

You can run a quick JSON shape check with:

```bash
cargo clippy
```

A panic at `src/pages/hub.rs` while deserializing the fixture means an unknown tag or platform variant was found. The error message lists all valid names.

## Conventions

- Keep card entries self-contained and deterministic.
- Use `null` for `icon_url` unless the user explicitly provides a URL.
- Timestamps should be in UTC (`Z`) and roughly within the current year.
