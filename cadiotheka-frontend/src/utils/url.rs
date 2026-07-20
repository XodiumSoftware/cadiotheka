/// Backend API origin.
///
/// In release builds the frontend is served from `cadiotheka.com` and talks
/// directly to `api.cadiotheka.com`. In debug builds Trunk proxies requests to
/// the local backend, so an empty origin is used to keep URLs relative.
const fn backend_origin() -> &'static str {
    if cfg!(debug_assertions) {
        ""
    } else {
        "https://api.cadiotheka.com"
    }
}

/// Builds a full backend URL from a route prefix and path.
fn backend_url(prefix: &str, path: &str) -> String {
    let base = format!("{}{prefix}", backend_origin());
    if path.starts_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    }
}

/// Returns the full URL for a backend API path (`/data/...`).
pub fn api_url(path: &str) -> String {
    backend_url("/data", path)
}

/// Returns the full URL for an auth endpoint (`/auth/...`).
pub fn auth_url(path: &str) -> String {
    backend_url("/auth", path)
}

/// Appends a safe `redirect_to` query parameter to a URL, using the current
/// browser location as the return target. Relative paths are used in release
/// builds; the full URL is used during local development so the backend can
/// send the browser back to the Trunk dev server.
pub fn encode_redirect_url(base: &str) -> String {
    let redirect_to = leptos::web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_else(|| "/".to_string());

    let encoded = urlencoding::encode(&redirect_to);
    format!("{base}?redirect_to={encoded}")
}

/// Returns the full URL for an OAuth login provider endpoint (`/login/...`).
pub fn login_url(provider: &str) -> String {
    backend_url("/login", provider)
}
