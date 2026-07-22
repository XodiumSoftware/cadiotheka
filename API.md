# Cadiotheka API

This document describes the HTTP endpoints exposed by `cadiotheka-backend`.
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

## Authentication

The backend uses signed session cookies. Include credentials on all
authenticated requests.

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/auth/me` | session | Returns `{ "account": <Account> }` or 401. |
| PUT | `/auth/me` | session | Updates the current account. Body: `{ "bio": string }`. |
| GET | `/auth/logout` | session | Clears the session and redirects safely. |
| GET | `/login/github` | - | Returns `{ "url": <OAuth URL> }`. Accepts `?redirect_to=...`. |
| GET | `/auth/github/callback` | - | GitHub OAuth callback. Sets session cookie. |
| GET | `/login/google` | - | Returns `{ "url": <OAuth URL> }`. Accepts `?redirect_to=...`. |
| GET | `/auth/google/callback` | - | Google OAuth callback. Sets session cookie. |

`redirect_to` must be a relative path starting with `/` or an allowed origin.

## Accounts

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/data/accounts` | - | List all accounts. |
| POST | `/data/accounts` | admin | Create a new account. |
| GET | `/data/accounts/:id` | - | Read a single account. |
| PUT | `/data/accounts/:id` | admin | Replace an account. |
| DELETE | `/data/accounts/:id` | admin | Delete an account. |
| GET | `/auth/linked-providers` | session | Returns `{ "providers": ["github", ...] }`. |
| DELETE | `/auth/linked-providers/:provider` | session | Unlink the given provider. |

## Projects

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| GET | `/data/projects` | - | List all projects. |
| POST | `/data/projects` | session | Create a project. |
| GET | `/data/projects/:id` | - | Read a single project. |
| PUT | `/data/projects/:id` | session owner/admin | Replace a project. |
| PATCH | `/data/projects/:id` | session owner/admin | Partially update a project. |
| DELETE | `/data/projects/:id` | session owner/admin | Delete a project. |
| POST | `/data/projects/:id/favorites` | session | Toggle favorite status for the current user. |
| POST | `/data/projects/:id/icon` | session owner/admin | Upload a project icon (`multipart/form-data`, field `icon`). |
| GET | `/data/icons/:project_id/:icon_id` | - | Serve a project icon. |

### Project payload limits

| Field | Limit |
|-------|-------|
| `title` | 100 characters |
| `description` | 500 characters |
| `extended_desc` | 5000 characters |
| `icon_url` key | 200 characters |
| Icon upload | 5 MiB, PNG/JPEG/WebP only |
