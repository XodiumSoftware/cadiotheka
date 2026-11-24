use worker::{Request, Response, Result};

/// Fallback origin used when a request carries no usable `Origin` header.
///
/// This is the production site, not an allowlist of the only sites permitted
/// to call the API: see [`select_allowed_origin`].
const ALLOWED_ORIGINS: &[&str] = &[
    "https://cadiotheka.com",
    "https://www.cadiotheka.com",
    "https://xodium.org",
    "https://www.xodium.org",
    "http://localhost:8080",
];

/// Adds CORS headers to a response so a frontend served from a different
/// origin can read the JSON body.
///
/// `origin` is the requesting page's own origin, which is echoed back. On its
/// own, an echoed origin grants no more than `*` would; together with
/// `Access-Control-Allow-Credentials`, though, it lets that origin read a
/// signed-in response. This is not an allowlist: treat any endpoint that acts
/// with the caller's session as reachable from any origin that calls it.
///
/// Returns the original response unchanged if its headers are immutable
/// (e.g. redirects created with `Response::redirect`). Propagates any other
/// header error so CORS misconfigurations are not silently ignored.
pub(crate) fn add_cors_headers(mut resp: Response, origin: &str) -> Result<Response> {
    let headers = resp.headers_mut();
    match apply_cors_headers(headers, origin) {
        Ok(()) => Ok(resp),
        Err(err) if is_immutable_headers_error(&err) => Ok(resp),
        Err(err) => Err(err),
    }
}

/// Writes the CORS response headers for `origin` into `headers`.
pub(crate) fn apply_cors_headers(headers: &worker::Headers, origin: &str) -> Result<()> {
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    headers.set("Access-Control-Allow-Credentials", "true")?;
    Ok(())
}

/// Heuristic to detect the immutable-headers error returned by `web_sys` when
/// trying to mutate a response with a guard such as a redirect.
fn is_immutable_headers_error(err: &worker::Error) -> bool {
    let message = err.to_string().to_lowercase();
    message.contains("immutable")
        || message.contains("guard")
        || message.contains("headers are immutable")
}

/// Responds to CORS preflight requests.
pub(crate) fn cors_preflight(origin: &str) -> Result<Response> {
    let mut resp = Response::empty()?;
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    headers.set("Access-Control-Allow-Credentials", "true")?;
    Ok(resp)
}

/// Returns the origin to echo in CORS headers for `req`.
///
/// The browser's `Origin` header is echoed back so any frontend can read the
/// public API, including a preview deployment or a local Trunk server. A
/// missing header (non-browser clients) falls back to the first entry of
/// [`ALLOWED_ORIGINS`], which is only ever read by clients that ignore CORS.
pub(crate) fn allowed_origin(req: &Request) -> String {
    select_allowed_origin(req.headers().get("Origin").ok().flatten().as_deref())
}

/// Selects the origin to echo from an optional request origin header value.
///
/// Only a syntactically valid `origin` (a scheme and a host, no path) is
/// echoed; anything else — a missing header, or a crafted value that would
/// produce a malformed header — falls back to the first allowed origin.
pub(crate) fn select_allowed_origin(origin: Option<&str>) -> String {
    origin
        .filter(|value| is_valid_origin(value))
        .unwrap_or(ALLOWED_ORIGINS[0])
        .to_string()
}

/// Whether `value` is shaped like a browser origin: `scheme://host[:port]`
/// with nothing after it.
fn is_valid_origin(value: &str) -> bool {
    let Ok(url) = url::Url::parse(value) else {
        return false;
    };
    matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some_and(|host| !host.is_empty())
        && url.path() == "/"
        && url.query().is_none()
        && url.fragment().is_none()
        && url.username().is_empty()
        && url.password().is_none()
}

/// Builds a generic 500 response with CORS headers so frontend error handlers
/// can read it when an unhandled worker error occurs.
///
/// The raw error is logged server-side and never sent to the client: error
/// strings can contain database details, OAuth response bodies, or other
/// internals that must not leak to browsers.
pub(crate) fn error_response_with_cors(err: &worker::Error, origin: &str) -> Result<Response> {
    worker::console_error!("unhandled request error: {err}");
    let resp = crate::utils::error_response("Internal Server Error", 500)?;
    add_cors_headers(resp, origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_origin_selects_production_origin() {
        assert_eq!(
            select_allowed_origin(Some("https://cadiotheka.com")),
            "https://cadiotheka.com"
        );
        assert_eq!(
            select_allowed_origin(Some("https://www.cadiotheka.com")),
            "https://www.cadiotheka.com"
        );
        assert_eq!(
            select_allowed_origin(Some("https://xodium.org")),
            "https://xodium.org"
        );
        assert_eq!(
            select_allowed_origin(Some("https://www.xodium.org")),
            "https://www.xodium.org"
        );
    }

    #[test]
    fn allowed_origin_selects_localhost() {
        assert_eq!(
            select_allowed_origin(Some("http://localhost:8080")),
            "http://localhost:8080"
        );
    }

    #[test]
    fn allowed_origin_falls_back_for_missing_origin() {
        assert_eq!(select_allowed_origin(None), "https://cadiotheka.com");
    }

    #[test]
    fn allowed_origin_echoes_any_well_formed_origin() {
        assert_eq!(
            select_allowed_origin(Some("https://preview.cadiotheka.pages.dev")),
            "https://preview.cadiotheka.pages.dev"
        );
        assert_eq!(
            select_allowed_origin(Some("http://192.168.1.10:8080")),
            "http://192.168.1.10:8080"
        );
    }

    #[test]
    fn allowed_origin_rejects_malformed_values() {
        assert_eq!(
            select_allowed_origin(Some("https://evil.example/path")),
            "https://cadiotheka.com"
        );
        assert_eq!(
            select_allowed_origin(Some("https://evil.example, https://cadiotheka.com")),
            "https://cadiotheka.com"
        );
        assert_eq!(
            select_allowed_origin(Some("null")),
            "https://cadiotheka.com"
        );
    }
}
