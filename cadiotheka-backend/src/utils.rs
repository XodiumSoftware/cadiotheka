use worker::{Request, Response, Result, RouteContext, wasm_bindgen};

/// KV binding used for rate-limit counters. Shared with the auth module so
/// only one KV namespace is required.
const RATE_LIMIT_KV_BINDING: &str = "AUTH";
/// Sliding window duration for rate limiting, in seconds.
const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;
/// Maximum requests allowed per window per client IP.
const RATE_LIMIT_MAX_REQUESTS: u32 = 30;

/// Builds a JSON error response with the given message and HTTP status.
///
/// The response body has the shape `{"error": "message"}`. This is used
/// instead of `Response::error` so the frontend can reliably parse error
/// payloads as JSON.
pub fn error_response(message: &str, status: u16) -> Result<Response> {
    Ok(Response::from_json(&serde_json::json!({ "error": message }))?.with_status(status))
}

/// Returns the best-effort client IP from Cloudflare/forwarded headers.
fn client_ip(req: &Request) -> String {
    req.headers()
        .get("CF-Connecting-IP")
        .ok()
        .flatten()
        .or_else(|| {
            req.headers()
                .get("X-Forwarded-For")
                .ok()
                .flatten()
                .and_then(|value| value.split(',').next().map(str::trim).map(String::from))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Checks whether the request is within the rate limit for the given namespace.
///
/// Returns `Ok(None)` when the request may proceed. Returns `Ok(Some(response))`
/// with a `429 Too Many Requests` response when the limit is exceeded.
pub async fn check_rate_limit(
    req: &Request,
    ctx: &RouteContext<()>,
    namespace: &str,
) -> Result<Option<Response>> {
    let ip = client_ip(req);
    let key = format!("rate_limit:{namespace}:{ip}");
    let kv = ctx.env.kv(RATE_LIMIT_KV_BINDING)?;

    let count: u32 = kv
        .get(&key)
        .text()
        .await?
        .and_then(|text| text.parse().ok())
        .unwrap_or(0);

    if count >= RATE_LIMIT_MAX_REQUESTS {
        return Ok(Some(error_response(
            "Rate limit exceeded. Please try again later.",
            429,
        )?));
    }

    let next = (count + 1).to_string();
    kv.put(&key, &next)?
        .expiration_ttl(RATE_LIMIT_WINDOW_SECONDS)
        .execute()
        .await?;

    Ok(None)
}

/// Wraps an error value into a `worker::Error::RustError`.
pub fn rust_err<E: std::fmt::Display>(err: E) -> worker::Error {
    worker::Error::RustError(err.to_string())
}

/// Converts an optional string into a D1-compatible `JsValue`.
///
/// D1 rejects JavaScript `undefined` values, which is what `Option::into`
/// currently produces for `None`. This helper emits an explicit SQL `NULL`
/// (`JsValue::NULL`) instead.
pub fn js_option(value: Option<String>) -> wasm_bindgen::JsValue {
    match value {
        Some(s) => s.into(),
        None => wasm_bindgen::JsValue::NULL,
    }
}

/// Returns the current UTC time using the JavaScript `Date` API.
///
/// `std::time` is unavailable in the Workers WASM runtime, so this is the
/// only way to obtain the current time in a Cloudflare Worker.
pub fn now_utc() -> time::OffsetDateTime {
    let millis = worker::js_sys::Date::now();
    time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds_f64(millis / 1_000.0)
}

/// Returns a public origin for the request, preferring the `X-Forwarded-Host`
/// and `X-Forwarded-Proto` headers used by Cloudflare, then falling back to
/// the request's own URL origin (including the port when present).
///
/// This is used to build OAuth redirect URLs that must match the registered
/// callback origin.
pub fn public_origin(req: &Request) -> String {
    let headers = req.headers();
    let host = headers
        .get("X-Forwarded-Host")
        .ok()
        .flatten()
        .or_else(|| {
            req.url().ok().map(|url| {
                let mut host = url.host_str().unwrap_or("").to_string();
                if let Some(port) = url.port() {
                    host.push(':');
                    host.push_str(&port.to_string());
                }
                host
            })
        })
        .unwrap_or_default();
    let proto = headers
        .get("X-Forwarded-Proto")
        .ok()
        .flatten()
        .unwrap_or_else(|| {
            req.url()
                .ok()
                .map_or_else(|| "https".to_string(), |url| url.scheme().to_string())
        });
    format!("{proto}://{host}")
}

/// Returns the value of a query parameter from a URL, if present.
pub fn query_param(url: &url::Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

/// Origins allowed for post-auth browser redirects. Must match the frontend
/// deployment origins. Relative paths are also accepted and resolved against
/// the request's public origin.
const ALLOWED_REDIRECT_ORIGINS: &[&str] = &["https://cadiotheka.com", "https://www.cadiotheka.com"];

/// Localhost origins allowed for non-HTTPS development requests.
const ALLOWED_LOCALHOST_ORIGINS: &[&str] = &["http://localhost:8080", "http://localhost:8787"];

/// Returns a safe redirect target from a query parameter.
///
/// Accepts:
/// - relative paths starting with `/` (returned as-is),
/// - absolute URLs whose origin is in `ALLOWED_REDIRECT_ORIGINS`,
/// - local dev origins (`http://localhost:8080`, `http://localhost:8787`) when
///   the request itself is not HTTPS.
///
/// URLs containing credentials, explicit ports, or protocol-relative URLs
/// are rejected. Any other value returns `None`.
pub fn safe_redirect_target(is_https: bool, url: &url::Url, param: &str) -> Option<String> {
    let target = query_param(url, param)?;
    if target.starts_with('/') && !target.starts_with("//") {
        return Some(target);
    }

    let parsed = url::Url::parse(&target).ok()?;
    let host = parsed.host_str()?;
    let origin = format!(
        "{}://{}{}",
        parsed.scheme(),
        host,
        format_port(parsed.port())
    );

    let is_localhost = host == "localhost";
    if parsed.username() != ""
        || parsed.password().is_some()
        || (!is_localhost && parsed.port().is_some())
    {
        return None;
    }

    if ALLOWED_REDIRECT_ORIGINS
        .iter()
        .any(|&allowed| allowed == origin)
    {
        return Some(target);
    }

    if !is_https
        && ALLOWED_LOCALHOST_ORIGINS
            .iter()
            .any(|&allowed| allowed == origin)
    {
        return Some(target);
    }

    None
}

fn format_port(port: Option<u16>) -> String {
    match port {
        Some(p) => format!(":{p}"),
        None => String::new(),
    }
}

/// Returns whether a request was made over HTTPS based on its URL scheme.
pub fn is_https_request(req: &Request) -> bool {
    req.url().ok().is_none_or(|u| u.scheme() == "https")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_redirect_target_accepts_relative_paths() {
        let url = url::Url::parse("https://api.cadiotheka.com/auth/logout?redirect_to=/projects")
            .unwrap();
        assert_eq!(
            safe_redirect_target(true, &url, "redirect_to"),
            Some("/projects".to_string())
        );
    }

    #[test]
    fn safe_redirect_target_accepts_allowed_origin() {
        let url = url::Url::parse(
            "https://api.cadiotheka.com/auth/logout?redirect_to=https://cadiotheka.com/",
        )
        .unwrap();
        assert_eq!(
            safe_redirect_target(true, &url, "redirect_to"),
            Some("https://cadiotheka.com/".to_string())
        );
    }

    #[test]
    fn safe_redirect_target_accepts_localhost_for_http_dev() {
        let url = url::Url::parse(
            "http://localhost:8787/auth/logout?redirect_to=http://localhost:8080/index.html%23dev",
        )
        .unwrap();
        assert_eq!(
            safe_redirect_target(false, &url, "redirect_to"),
            Some("http://localhost:8080/index.html#dev".to_string())
        );
    }

    #[test]
    fn safe_redirect_target_rejects_protocol_relative_url() {
        let url = url::Url::parse("https://api.cadiotheka.com/auth/logout?redirect_to=//evil.com")
            .unwrap();
        assert_eq!(safe_redirect_target(true, &url, "redirect_to"), None);
    }

    #[test]
    fn safe_redirect_target_rejects_unknown_origin() {
        let url =
            url::Url::parse("https://api.cadiotheka.com/auth/logout?redirect_to=https://evil.com/")
                .unwrap();
        assert_eq!(safe_redirect_target(true, &url, "redirect_to"), None);
    }

    #[test]
    fn safe_redirect_target_rejects_localhost_over_https() {
        let url = url::Url::parse(
            "https://api.cadiotheka.com/auth/logout?redirect_to=http://localhost:8080/",
        )
        .unwrap();
        assert_eq!(safe_redirect_target(true, &url, "redirect_to"), None);
    }

    #[test]
    fn safe_redirect_target_rejects_url_with_port() {
        let url =
            url::Url::parse("http://localhost:8787/auth/logout?redirect_to=http://localhost:9999/")
                .unwrap();
        assert_eq!(safe_redirect_target(false, &url, "redirect_to"), None);
    }

    #[test]
    fn safe_redirect_target_rejects_url_with_credentials() {
        let url = url::Url::parse(
            "http://localhost:8787/auth/logout?redirect_to=http://user:pass@localhost:8080/",
        )
        .unwrap();
        assert_eq!(safe_redirect_target(false, &url, "redirect_to"), None);
    }

    #[test]
    fn safe_redirect_target_returns_none_for_missing_param() {
        let url = url::Url::parse("https://api.cadiotheka.com/auth/logout").unwrap();
        assert_eq!(safe_redirect_target(true, &url, "redirect_to"), None);
    }
}
