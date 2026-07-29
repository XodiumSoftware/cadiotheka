# Turnstile Spin skill for Cadiotheka

Inlined Cloudflare Turnstile integration skill tailored for the Cadiotheka
Leptos CSR + Cloudflare Pages Functions Rust backend project.

## When to load

- "Add Turnstile", "set up CAPTCHA", "protect a form or endpoint"
- Mentions of bot protection on project submission or IFC downloads

## Detection

- Frontend: Rust + Leptos CSR (`cadiotheka-frontend/src/components/ui/modals/*.rs`)
- Backend: Cloudflare Pages Functions in `cadiotheka-backend/src/api/*.rs`
- Existing CAPTCHA: look for `cf-turnstile`, `g-recaptcha`, `h-captcha`, `siteverify`

## Insertion points

1. Add Project form → `POST /data/projects`
2. Download IFC → `POST /data/projects/:id/downloads`

## Frontend contract

- Load `<script src="https://challenges.cloudflare.com/turnstile/v0/api.js" async defer></script>` in `index.html`.
- Render a `<div class="cf-turnstile" data-sitekey="<SITEKEY>" data-action="turnstile-spin-v2"></div>` in each protected modal.
- Read the token from the hidden `<input name="cf-turnstile-response">` that the widget creates.
- Pass the token to the backend via the `X-Turnstile-Token` request header in the existing request helpers.
- Reset the widget after any non-redirect error: `window.turnstile?.reset()` or use the provided `reset_turnstile()` helper.

## Backend contract

- Read the token from the request body or header.
- Call `POST https://challenges.cloudflare.com/turnstile/v0/siteverify` with form-encoded `secret`, `response`, and `remoteip`.
- Gate on `success === true`; otherwise return 403.
- Reference the secret from the environment as `TURNSTILE_SECRET` (`ctx.env.secret("TURNSTILE_SECRET")` in Rust Workers).

## Validation

- Run `cargo test` and `cargo clippy` on both crates.
- Verify the Turnstile script is present in `index.html`.
- Verify `data-action="turnstile-spin-v2"` on every widget div.
- (Runtime) Submit a dummy token to the backend and check `invalid-input-response` from siteverify.
