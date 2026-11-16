# TODO — Cloudflare / Wrangler backlog

Candidate Cloudflare features for the backend worker, beyond what is already
wired up in `backend/wrangler.toml` (D1, KV, Rate Limiting, R2, Observability).

| # | Feature | Wrangler config | Why it helps | Effort | Priority |
|---|---------|-----------------|--------------|--------|----------|
| 1 | Queues for async GLB conversion | `[[queues.producers]]`, `[[queues.consumers]]`, `queue()` handler in `lib.rs` | Removes CPU-limit risk from synchronous `try_export_glb` in `convert_project_glb`; adds retries and makes uploads feel instant | Medium | ✅ Done |
| 2 | Cron trigger for maintenance sweeps | `[triggers] crons = [...]`, `#[event(scheduled)]` handler | Purge expired KV sessions, reap orphaned R2 objects after deletes, recompute download/favorite counters | Low | High |
| 3 | Edge caching for JSON reads | Cache API (`cache.match` / `cache.put`) + ETag revalidation | Cuts D1 read volume and latency on public list endpoints such as `GET /data/projects` | Low | Medium |
| 4 | R2-backed avatars and thumbnails | Reuse existing `PROJECT_ASSETS` bucket (or a new one) | Removes dependence on GitHub/Google CDN URLs and enables caching project thumbnails | Medium | Medium |
| 5 | Image transforms for avatars/thumbnails | `cf.image` fetch options or Cloudflare Images | Generates resized variants on the fly instead of storing multiple sizes | Medium | Low |
| 6 | Transactional email | `[[send_email]]` binding / Email Workers | Verification emails, notification digests, moderation alerts | Medium | Low |
| 7 | Semantic search over projects | Vectorize binding | Search beyond tag filtering, across descriptions and IFC metadata | High | Low |
| 8 | Real-time collaboration | Durable Objects binding | Live editing and presence, if that becomes a product goal | High | Low |
| 9 | Review compatibility flags | `compatibility_flags = ["nodejs_compat"]` | Unblocks Node-built-in-dependent libraries if one is ever needed | Low | Low |

## Recommended order

1. ~~**Queue-based GLB conversion** — removes a genuine CPU-limit failure mode.~~ ✅ Implemented via the `GLB_CONVERSION` queue.
2. **Daily cron sweep** — cheap, prevents silent KV/R2 cost creep.
3. **Edge caching for JSON reads** — low effort, measurable latency win.
