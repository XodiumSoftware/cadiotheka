<div id="readme-top"></div>

<h1 align="center">
  <br />
    <a href="https://xodium.org/">
        <img src="logo.svg" alt="Cadiotheka Logo" width="200">
    </a>
  <br /><br />
  Cadiotheka
  <br />
  <br />
</h1>

<h4 align="center">The open hub for CAD creators</h4><br />

<div align="center">

[![Contributors][contributors_shield_url]][contributors_url]
[![Issues][issues_shield_url]][issues_url]
[![License][license_shield_url]][license_url]
</div>

## Table of Contents

- [About](#about)
- [Usage](#usage)
- [Built With](#built-with)
- [Code of Conduct][code_of_conduct_url]
- [Contributing][contributing_url]
- [License][license_url]
- [Requirements](#requirements)
- [Security][security_url]

## About

Cadiotheka is an open hub for CAD creators. It collects, organizes, and provides resources, tooling, and references to support people working with computer-aided design. The hub runs as a browser application built with [leptos](https://github.com/leptos) and compiled to WebAssembly, backed by a Cloudflare Pages Functions Rust backend that uses a D1 database and an R2 bucket for uploaded project icons.

## Requirements

- [Rust](https://www.rust-lang.org/) — latest stable toolchain
- `wasm32-unknown-unknown` target (for the frontend)
- [Trunk](https://trunkrs.dev/) (for the frontend)
- [Node.js](https://nodejs.org/) and `npx` (for the backend)
- A [Cloudflare](https://cloudflare.com/) account with D1 and R2 access (for backend deployment)

Install the target and Trunk with:

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

<p align="right"><a href="#readme-top">▲</a></p>

## Running Locally

Cadiotheka is a Cargo workspace with two members: `cadiotheka-frontend` and `cadiotheka-backend`.

### Frontend

Start a local Trunk dev server:

```bash
git clone https://github.com/XodiumSoftware/cadiotheka.git
cd cadiotheka
cd cadiotheka-backend
npx wrangler dev
```

In a second terminal, start the frontend:

```bash
cd cadiotheka-frontend
trunk serve --port 8080
```

Then open <http://localhost:8080/index.html#dev> in a browser.

Trunk proxies `/data/*` requests to the backend dev server on `http://127.0.0.1:8787`.

### Backend

The backend is a Cloudflare Pages Functions Rust worker. First build the WASM bundle, then run Wrangler:

```bash
cd cadiotheka-backend
cargo install worker-build --version 0.7.5 --force
worker-build
npx wrangler dev
```

The backend API is available at <http://localhost:8787/data/accounts> by default.

Project icons are uploaded through the backend and stored in the `CADIOTHEKA_PROJECTS_ICONS` R2 bucket. The database stores only the generated R2 object key (for example `icons/<project_id>/<uuid>`), and the frontend renders icons through the backend icon route.

To create the local D1 database tables:

```bash
cd cadiotheka-backend
npx wrangler d1 execute cadiotheka-db --file=schemas/accounts.sql --local
npx wrangler d1 execute cadiotheka-db --file=schemas/projects.sql --local
```

For icon uploads, also create or bind an R2 bucket in `wrangler.toml`:

```toml
[[r2_buckets]]
binding = "CADIOTHEKA_PROJECTS_ICONS"
bucket_name = "cadiotheka-projects-icons"
```

Create accounts and projects through the application UI or API as needed.

### Tests & Linting

Run the full workspace test suite:

```bash
cargo test
```

Lint each crate:

```bash
cd cadiotheka-frontend
cargo clippy --target wasm32-unknown-unknown --all-targets --all-features -- -D warnings
cd ../cadiotheka-backend
cargo clippy --all-targets --all-features -- -D warnings
```

### Frontend Build

For a release build:

```bash
cd cadiotheka-frontend
trunk build --release
```

The static site is placed in `cadiotheka-frontend/dist/`.

## Backend Deployment

1. Create a D1 database:
   ```bash
   npx wrangler d1 create cadiotheka-db
   ```

2. Update `cadiotheka-backend/wrangler.toml` with the database ID from step 1.

3. Create a KV namespace for OAuth state and sessions:
   ```bash
   npx wrangler kv:namespace create AUTH_KV
   ```

   Then copy the resulting ID into `cadiotheka-backend/wrangler.toml` under `[[kv_namespaces]]`.

4. Configure secrets for OAuth and session signing:
   ```bash
   npx wrangler secret put SESSION_SECRET
   npx wrangler secret put GITHUB_CLIENT_ID
   npx wrangler secret put GITHUB_CLIENT_SECRET
   npx wrangler secret put GOOGLE_CLIENT_ID
   npx wrangler secret put GOOGLE_CLIENT_SECRET
   ```

   - `SESSION_SECRET` can be any long random string.
   - GitHub and Google credentials come from OAuth apps registered at:
     - GitHub: `https://github.com/settings/developers`
     - Google: `https://console.cloud.google.com/apis/credentials`
   - Use callback URLs `https://api.cadiotheka.com/auth/github/callback` and `https://api.cadiotheka.com/auth/google/callback`.
   - If your backend is served from the same origin as the frontend (e.g. via a Cloudflare Pages Function or a zone route), use `https://cadiotheka.com/auth/...` instead.

5. Apply the schema:
   ```bash
   npx wrangler d1 execute cadiotheka-db --file=cadiotheka-backend/schemas/accounts.sql
   npx wrangler d1 execute cadiotheka-db --file=cadiotheka-backend/schemas/projects.sql
   ```

   Create the first accounts and projects through the deployed application UI or API.

6. Build and deploy:
   ```bash
   npx wrangler deploy
   ```

<p align="right"><a href="#readme-top">▲</a></p>

## Built With

<div align="center">

[![Built With][built_with_shield_url]][built_with_url]
</div>

<p align="right"><a href="#readme-top">▲</a></p>

[code_of_conduct_url]: https://github.com/XodiumSoftware/cadiotheka?tab=coc-ov-file

[contributing_url]: https://github.com/XodiumSoftware/cadiotheka/blob/main/CONTRIBUTING.md

[contributors_shield_url]: https://img.shields.io/github/contributors/XodiumSoftware/cadiotheka?style=for-the-badge&color=blue

[contributors_url]: https://github.com/XodiumSoftware/cadiotheka/graphs/contributors

[issues_shield_url]: https://img.shields.io/github/issues/XodiumSoftware/cadiotheka?style=for-the-badge&color=yellow

[issues_url]: https://github.com/XodiumSoftware/cadiotheka/issues

[license_shield_url]: https://img.shields.io/github/license/XodiumSoftware/cadiotheka?style=for-the-badge&color=blue

[license_url]: https://github.com/XodiumSoftware/cadiotheka?tab=AGPL-3.0-1-ov-file

[built_with_shield_url]: https://skillicons.dev/icons?i=rust,github,githubactions

[built_with_url]: https://skillicons.dev

[security_url]: https://github.com/XodiumSoftware/cadiotheka?tab=security-ov-file
