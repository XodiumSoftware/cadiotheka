# Cadiotheka API

This document describes the HTTP endpoints exposed by `backend`.
All `/data/*` and `/auth/*` responses include CORS headers for the frontend.

## Error format

Generic errors return JSON with a single `error` field:

```json
{ "error": "Not found" }
```

Validation errors for project creation/replacement return a field map:

```json
{ "errors": { "title": "Title must be 100 characters or fewer" } }
```

Unauthenticated requests to session-protected endpoints return `401` and
rate-limited endpoints return `429`, both with the same `{ "error": ... }`
shape. Unexpected failures return `500` with a generic message; internal
details are logged server-side and never exposed.

## Authentication

The backend uses signed session cookies. Include credentials on all
authenticated requests.

| Method | Path                          | Auth    | Description                                                                                                                                           |
| ------ | ----------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| GET    | `/auth/me`                    | session | Returns `{ "account": <Account> }` or 401. The only endpoint that includes the account's private `email` and `viewer_preferences` fields.             |
| PUT    | `/auth/me`                    | session | Updates the current account. Body: `{ "bio"?: string, "viewer_preferences"?: string }`. Bio is limited to 160 characters, viewer preferences to 4096. |
| GET    | `/auth/me/viewer-preferences` | session | Returns `{ "viewer_preferences": string }` for the current account.                                                                                   |
| GET    | `/auth/logout`                | session | Clears the session and redirects safely.                                                                                                              |
| GET    | `/login/github`               | -       | Returns `{ "url": <OAuth URL> }`. Accepts `?redirect_to=...`.                                                                                         |
| GET    | `/auth/github/callback`       | -       | GitHub OAuth callback. Sets session cookie.                                                                                                           |
| GET    | `/login/google`               | -       | Returns `{ "url": <OAuth URL> }`. Accepts `?redirect_to=...`.                                                                                         |
| GET    | `/auth/google/callback`       | -       | Google OAuth callback. Sets session cookie.                                                                                                           |

`redirect_to` must be a relative path starting with `/` or an allowed origin.

## Accounts

Public account responses omit the private `email` and `viewer_preferences`
fields; they are only included in `/auth/me` for the account owner.

| Method | Path                               | Auth    | Description                                                                                                |
| ------ | ---------------------------------- | ------- | ---------------------------------------------------------------------------------------------------------- |
| GET    | `/data/accounts`                   | -       | List accounts (public fields only), oldest first. Accepts `?limit=` (default 100, max 500) and `?offset=`. |
| POST   | `/data/accounts`                   | admin   | Create a new account.                                                                                      |
| GET    | `/data/accounts/:id`               | -       | Read a single account (public fields only).                                                                |
| PUT    | `/data/accounts/:id`               | admin   | Replace an account.                                                                                        |
| DELETE | `/data/accounts/:id`               | admin   | Delete an account.                                                                                         |
| GET    | `/auth/linked-providers`           | session | Returns `{ "providers": ["github", ...] }`.                                                                |
| DELETE | `/auth/linked-providers/:provider` | session | Unlink the given provider. The account's sole remaining provider cannot be unlinked.                       |

## Metadata

Tags are hardcoded as an enum in `frontend/src/metadata/tags.rs`. Project rows
store tag wire ids as JSON arrays and the frontend resolves labels and colors
from the enum.

## Projects

The server assigns `id`, `timestamp`, `downloads`, and `favorites` on creation
and preserves them on replacement; values for those fields in request bodies
are ignored. `POST /data/projects` additionally requires a Cloudflare Turnstile
token in the `X-Turnstile-Token` header.

| Method | Path                                      | Auth                | Description                                                                                                                                  |
| ------ | ----------------------------------------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| GET    | `/data/projects`                          | -                   | List projects, newest first. Accepts `?limit=` (default 100, max 500) and `?offset=`.                                                        |
| POST   | `/data/projects`                          | session + Turnstile | Create a project. Returns the created project.                                                                                               |
| GET    | `/data/projects/:id`                      | -                   | Read a single project.                                                                                                                       |
| PUT    | `/data/projects/:id`                      | session owner/admin | Replace a project's editable fields.                                                                                                         |
| PATCH  | `/data/projects/:id`                      | session owner/admin | Partially update a project (`title`, `tags`, `collaborator_ids`, `description`).                                                             |
| DELETE | `/data/projects/:id`                      | session owner/admin | Delete a project.                                                                                                                            |
| POST   | `/data/projects/:id/favorites`            | session             | Toggle favorite status for the current user. Returns the updated project.                                                                    |
| POST   | `/data/projects/:id/downloads`            | -                   | Increment the project's download counter and return the updated project.                                                                     |
| POST   | `/data/projects/:id/ifc`                  | session owner/admin | Upload an IFC model (`multipart/form-data`, field `ifc`; optional field `version`, default `1.0.0`). Creates a version in state `undefined`. |
| DELETE | `/data/projects/:id/ifc`                  | session owner/admin | Delete all IFC versions of the project. Returns `{ "deleted": true }`.                                                                       |
| GET    | `/data/projects/:id/versions`             | -                   | List the project's versions. Versions in state `undefined` are only included for project editors.                                            |
| PATCH  | `/data/projects/:id/versions/:version_id` | session owner/admin | Update a version's state. Body: `{ "state": "undefined" \| "alpha" \| "beta" \| "stable" }`.                                                 |
| DELETE | `/data/projects/:id/versions/:version_id` | session owner/admin | Delete a single version.                                                                                                                     |
| GET    | `/data/projects/:id/glb`                  | -                   | Serve the project's converted GLB (`model/gltf-binary`). Converts on demand when not cached.                                                 |
| POST   | `/data/projects/:id/glb`                  | session owner/admin | Convert the latest IFC version to GLB. Returns `{ "status": "ready" }`, or 422 when the model has no renderable geometry.                    |
| GET    | `/data/projects/:id/glb-metadata`         | -                   | Serve per-primitive metadata (JSON) mapping GLB primitives to IFC entities: express id, GlobalId, name, IFC type, and IFC material names.      |
| GET    | `/data/ifcs/:version_id/:filename`        | -                   | Download a project IFC model as an attachment and increment its version download counter.                                                    |

### Project payload limits

| Field         | Limit                             |
| ------------- | --------------------------------- |
| `title`       | 100 characters                    |
| `description` | 5000 characters                   |
| IFC upload    | 25 MiB, `.ifc` extension required |
