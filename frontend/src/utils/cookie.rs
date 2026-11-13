//! Browser cookie reading for non-`HttpOnly` cookies set by the backend.
//!
//! The session cookie is `HttpOnly` and therefore invisible to the frontend.
//! The backend also sets [`AUTH_SESSION_HINT_COOKIE`], a JavaScript-readable
//! cookie that only states whether a session exists, so the app can skip the
//! `/auth/me` request for anonymous visitors.

use shared::routes::AUTH_SESSION_HINT_COOKIE;
use wasm_bindgen::JsCast;

/// Name of the backend session cookie on HTTPS.
///
/// Mirrors the backend's `__Host-`-prefixed name. `__Host-` cookies require
/// `Secure`, `Path=/`, and no `Domain`, so they are only set on HTTPS.
pub const SESSION_COOKIE_NAME: &str = "__Host-session";

/// Name of the backend session cookie on plain HTTP, used by local dev.
pub const SESSION_COOKIE_NAME_PLAIN: &str = "session";

/// Returns the value of the first cookie with the given name.
///
/// Returns `None` when the cookie is absent, when the cookie jar cannot be
/// read, or when the value is not valid ASCII.
pub fn cookie_value(name: &str) -> Option<String> {
    let cookies = html_document()?.cookie().ok()?;
    parse_cookie_value(&cookies, name)
}

/// Replaces the browser's cookie jar string (`document.cookie = ...`).
///
/// Assigning a value for an existing cookie overwrites it, so an expired
/// cookie (`Max-Age=0`) removes it.
pub(crate) fn set_document_cookie(value: &str) {
    if let Some(document) = html_document() {
        let _ = document.set_cookie(value);
    }
}

/// The `document` as an [`web_sys::HtmlDocument`], which owns the cookie API.
fn html_document() -> Option<web_sys::HtmlDocument> {
    let document = web_sys::window()?.document()?;
    let html: &web_sys::HtmlDocument = document.unchecked_ref();
    Some(html.clone())
}

/// Returns whether the browser holds a backend session cookie.
///
/// An absent hint means the visitor has no session cookie, so the frontend can
/// skip `/auth/me` instead of provoking a 401.
pub fn has_session_hint() -> bool {
    cookie_value(AUTH_SESSION_HINT_COOKIE).is_some()
}

/// Finds `name` in a `document.cookie` string and returns its raw value.
pub(crate) fn parse_cookie_value(jar: &str, name: &str) -> Option<String> {
    cookie::Cookie::split_parse(jar)
        .filter_map(Result::ok)
        .find(|cookie| cookie.name() == name)
        .map(|cookie| cookie.value().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cookie_value_finds_the_hint_among_other_cookies() {
        let jar = "theme=dark; __Host-session-hint=1; other=value";
        assert_eq!(
            parse_cookie_value(jar, AUTH_SESSION_HINT_COOKIE).as_deref(),
            Some("1")
        );
    }

    #[test]
    fn parse_cookie_value_returns_none_when_absent() {
        assert_eq!(
            parse_cookie_value("theme=dark", AUTH_SESSION_HINT_COOKIE),
            None
        );
    }

    #[test]
    fn hint_cookie_name_does_not_collide_with_the_session_cookie() {
        assert_ne!(AUTH_SESSION_HINT_COOKIE, SESSION_COOKIE_NAME);
        assert_ne!(AUTH_SESSION_HINT_COOKIE, SESSION_COOKIE_NAME_PLAIN);
    }
}
