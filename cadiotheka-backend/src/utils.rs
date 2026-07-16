use worker::*;

/// Returns the current UTC time using the JavaScript `Date` API.
///
/// `std::time` is unavailable in the Workers WASM runtime, so this is the
/// only way to obtain the current time in a Cloudflare Worker.
pub fn now_utc() -> time::OffsetDateTime {
    let millis = worker::js_sys::Date::now();
    let seconds = (millis / 1_000.0) as i64;
    let nanos = ((millis % 1_000.0) * 1_000_000.0) as i32;
    time::OffsetDateTime::from_unix_timestamp(seconds).unwrap_or(time::OffsetDateTime::UNIX_EPOCH)
        + time::Duration::nanoseconds(nanos.into())
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
                .map(|url| url.scheme().to_string())
                .unwrap_or_else(|| "https".to_string())
        });
    format!("{proto}://{host}")
}

/// Returns the value of a query parameter from a URL, if present.
pub fn query_param(url: &url::Url, name: &str) -> Option<String> {
    url.query_pairs()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.into_owned())
}

/// Wraps an error value into a `worker::Error::RustError`.
pub fn rust_err<E: std::fmt::Display>(err: E) -> worker::Error {
    worker::Error::RustError(err.to_string())
}
