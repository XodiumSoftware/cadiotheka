# Rust library replacement opportunities

This list audits custom code in the Cadiotheka workspace that could be replaced
by established Rust crates.

| # | Replacement | File(s) | Effort | Benefit |
|---|-------------|---------|--------|---------|
| 1 | Sanitized markdown renderer (`ammonia` + `pulldown-cmark`) | `cadiotheka-frontend/src/components/ui/markdown.rs` | Low | High | ✅ Done |
| 2 | `html_escape` crate | `cadiotheka-frontend/src/components/ui/markdown.rs`, `cadiotheka-backend/src/api/auth.rs` | Low | Medium | ✅ Done |
| 3 | `timeago` / `humantime` | `cadiotheka-frontend/src/utils/format.rs` | Low | Medium | ✅ Done |
| 4 | `human_format` / `num-format` | `cadiotheka-frontend/src/utils/format.rs` | Low | Low-Medium | ✅ Done |
| 5 | `imagesize` crate | `cadiotheka-backend/src/api/projects.rs` | Low | Medium |
| 6 | `cookie` crate | `cadiotheka-backend/src/api/session.rs` | Medium | Medium |
| 7 | `serde_qs` | `cadiotheka-backend/src/utils.rs` | Low | Low |
| 8 | OIDC discovery (`openidconnect`) | `cadiotheka-backend/src/api/auth.rs` | Medium | Low |

## Notes

1. **Markdown rendering** is the closest match to the original question. The
   current implementation is a hand-written event loop over `pulldown-cmark` that
   emits styled HTML and manually escapes output. Replacing it with a sanitized
   renderer removes ~80 lines of fragile tag matching, fixes edge cases, and
   improves XSS resistance.

2. **HTML escaping** is currently implemented twice, slightly differently, in
   the frontend and backend. A dedicated crate removes the duplication and
   avoids subtle escaping bugs.

3. **Relative time formatting** uses threshold-based custom logic. A dedicated
   crate is smaller, localizable, and removes tests that must be maintained.
   Check default features for WASM bundle impact.

4. **Number formatting** for compact SI suffixes and thousands separators can be
   handled by small formatting crates. Verify WASM feature flags before adding.

5. **Image dimension sniffing** is currently hand-rolled for PNG/JPEG and cannot
   read WebP dimensions. `imagesize` is a tiny, header-only alternative that
   supports WebP.

6. **Cookie handling** is currently built with string formatting and manual
   `Cookie` header splitting. The `cookie` crate provides correct encoding and
   attribute handling.

7. **Query parameter parsing** is currently manual. `serde_qs` becomes useful if
   the API starts accepting more complex query structures.

8. **OIDC discovery** only makes sense if more OAuth providers are added; with
   only GitHub and Google it adds more complexity than it removes.

## Recommendation

Start with items 1 and 2. They remove the most custom code, improve security,
and have minimal dependency cost since `pulldown-cmark` is already used.
