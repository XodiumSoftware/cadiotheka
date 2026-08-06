use worker::{Headers, Request, Response, ResponseBody, ResponseBuilder, Result};

/// Origins allowed to call the API from a browser.
///
/// `http://localhost:8080` is included so developers can hit the local backend
/// directly (not only through Trunk's proxy) during local development. It is
/// never sent by a production deployment.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://cadiotheka.com",
    "https://www.cadiotheka.com",
    "http://localhost:8080",
];

/// Adds CORS headers to a response so the frontend (served from a different
/// origin) can read the JSON body.
///
/// Returns the original response unchanged if its headers are immutable
/// (e.g. redirects created with `Response::redirect`). Propagates any other
/// header error so CORS misconfigurations are not silently ignored.
pub fn add_cors_headers(mut resp: Response, origin: &str) -> Result<Response> {
    let headers = resp.headers_mut();
    if let Err(err) = headers.set("Access-Control-Allow-Origin", origin) {
        if is_immutable_headers_error(&err) {
            return Ok(resp);
        }
        return Err(err);
    }
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    headers.set("Access-Control-Allow-Credentials", "true")?;
    Ok(resp)
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
pub fn cors_preflight(origin: &str) -> Result<Response> {
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

/// Returns the request origin if it is in the allowed list, otherwise the
/// first allowed origin as a safe fallback.
pub fn allowed_origin(req: &Request) -> String {
    select_allowed_origin(req.headers().get("Origin").ok().flatten().as_deref())
}

/// Selects an allowed origin from an optional request origin header value.
///
/// If the origin is in [`ALLOWED_ORIGINS`] it is returned verbatim; otherwise
/// the first allowed origin is returned as a safe default.
pub fn select_allowed_origin(origin: Option<&str>) -> String {
    origin
        .and_then(|value| {
            ALLOWED_ORIGINS
                .iter()
                .find(|&&allowed| allowed == value)
                .map(|_| value)
        })
        .unwrap_or_else(|| ALLOWED_ORIGINS[0])
        .to_string()
}

/// Builds a raw 500 response with CORS headers so frontend error handlers can
/// read it when an unhandled worker error occurs.
pub fn error_response_with_cors(err: &worker::Error, origin: &str) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Content-Type", "text/plain")?;
    let _ = headers.set("Access-Control-Allow-Origin", origin);
    let _ = headers.set("Access-Control-Allow-Credentials", "true");
    let _ = headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    );
    let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
    Ok(ResponseBuilder::new()
        .with_status(500)
        .with_headers(headers)
        .body(ResponseBody::Body(err.to_string().into())))
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
    }

    #[test]
    fn allowed_origin_selects_localhost() {
        assert_eq!(
            select_allowed_origin(Some("http://localhost:8080")),
            "http://localhost:8080"
        );
    }

    #[test]
    fn allowed_origin_falls_back_for_missing_or_unknown_origin() {
        assert_eq!(select_allowed_origin(None), "https://cadiotheka.com");
        assert_eq!(
            select_allowed_origin(Some("https://evil.com")),
            "https://cadiotheka.com"
        );
    }
}
