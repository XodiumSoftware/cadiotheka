use base64::Engine as _;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use worker::*;

use crate::api::accounts::{Account, fetch_account_by_provider};
use crate::utils::{is_https_request, public_origin, rust_err, safe_redirect_target};

const AUTH_KV_BINDING: &str = "AUTH_KV";
const SESSION_COOKIE_NAME_PREFIX: &str = "__Host-session";
const SESSION_COOKIE_NAME_PLAIN: &str = "session";
const SESSION_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// Stored session metadata, keyed by session id in KV.
#[derive(Debug, Serialize, Deserialize)]
struct SessionData {
    account_id: String,
    provider: String,
    provider_id: String,
}

/// A signed session cookie value.
#[derive(Debug, Serialize, Deserialize)]
struct SessionCookie {
    id: String,
    sig: String,
}

fn kv(ctx: &RouteContext<()>) -> Result<KvStore> {
    ctx.env.kv(AUTH_KV_BINDING)
}

fn session_secret(ctx: &RouteContext<()>) -> Result<String> {
    Ok(ctx.env.secret("SESSION_SECRET")?.to_string())
}

type HmacSha256 = Hmac<Sha256>;

fn sign(secret: &str, id: &str) -> Result<String> {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(rust_err)?;
    mac.update(id.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

fn verify_signature(secret: &str, id: &str, sig: &str) -> Result<bool> {
    let expected = sign(secret, id)?;
    Ok(expected == sig)
}

/// Returns the cookie name to use based on whether the request is HTTPS.
///
/// The `__Host-` prefix requires `Secure`, so it can only be used on HTTPS.
/// Plain HTTP (local dev) falls back to the unprefixed name without `Secure`.
fn session_cookie_name(is_https: bool) -> &'static str {
    if is_https {
        SESSION_COOKIE_NAME_PREFIX
    } else {
        SESSION_COOKIE_NAME_PLAIN
    }
}

fn is_https_origin(origin: &str) -> bool {
    origin.starts_with("https://")
}

/// Builds a `Set-Cookie` header value for a session.
///
/// HTTPS uses `SameSite=None` so the cookie is sent to the API subdomain from
/// the frontend origin. HTTP (local dev) uses `SameSite=Lax`.
fn build_session_cookie(name: &str, encoded: &str, is_https: bool) -> String {
    if is_https {
        format!(
            "{name}={encoded}; Max-Age={SESSION_TTL_SECONDS}; Path=/; HttpOnly; Secure; SameSite=None"
        )
    } else {
        format!("{name}={encoded}; Max-Age={SESSION_TTL_SECONDS}; Path=/; HttpOnly; SameSite=Lax")
    }
}

/// Builds a cookie-clearing `Set-Cookie` header value.
fn build_clear_cookie(name: &str, is_https: bool) -> String {
    if is_https {
        format!("{name}=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=None")
    } else {
        format!("{name}=; Max-Age=0; Path=/; HttpOnly; SameSite=Lax")
    }
}

/// Creates a session for the given account and returns a `Set-Cookie` header
/// value that the caller can attach to a redirect response.
pub async fn create_session(
    ctx: &RouteContext<()>,
    account: &Account,
    req: &Request,
) -> Result<String> {
    let is_https = is_https_origin(&public_origin(req));
    let cookie_name = session_cookie_name(is_https);

    let secret = session_secret(ctx)?;
    let id = uuid::Uuid::new_v4().to_string();
    let sig = sign(&secret, &id)?;
    let data = SessionData {
        account_id: account.id.clone(),
        provider: account.provider.clone(),
        provider_id: account.provider_id.clone(),
    };

    kv(ctx)?
        .put(&id, &serde_json::to_string(&data).map_err(rust_err)?)?
        .expiration_ttl(SESSION_TTL_SECONDS)
        .execute()
        .await?;

    let cookie = serde_json::to_string(&SessionCookie { id, sig }).map_err(rust_err)?;
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(cookie);
    let value = build_session_cookie(cookie_name, &encoded, is_https);
    console_log!(
        "create_session: origin={} name={} cookie={}",
        public_origin(req),
        cookie_name,
        value
    );
    Ok(value)
}

/// Reads the session cookie from the request, validates its signature, looks it
/// up in KV, and returns the associated account. Returns `None` if there is
/// no session or it is invalid/expired.
pub async fn read_session(req: &Request, ctx: &RouteContext<()>) -> Result<Option<Account>> {
    let cookie_header = match req.headers().get("Cookie")? {
        Some(value) => value,
        None => {
            console_log!(
                "read_session: no Cookie header present (origin={})",
                public_origin(req)
            );
            return Ok(None);
        }
    };

    let is_https = is_https_origin(&public_origin(req));
    let cookie_name = session_cookie_name(is_https);
    console_log!(
        "read_session: public_origin={} is_https={} cookie_name={}",
        public_origin(req),
        is_https,
        cookie_name
    );

    let encoded = cookie_header
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.strip_prefix(cookie_name))
        .filter_map(|rest| rest.strip_prefix('='))
        .next();

    let encoded = match encoded {
        Some(value) => value,
        None => {
            console_log!(
                "read_session: cookie {} not found in: {}",
                cookie_name,
                cookie_header
            );
            return Ok(None);
        }
    };

    let decoded = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded) {
        Ok(bytes) => bytes,
        Err(_) => {
            console_log!("read_session: failed to base64-decode session cookie");
            return Ok(None);
        }
    };

    let cookie: SessionCookie = match serde_json::from_slice(&decoded) {
        Ok(cookie) => cookie,
        Err(_) => {
            console_log!("read_session: failed to parse session cookie JSON");
            return Ok(None);
        }
    };

    let secret = session_secret(ctx)?;
    if !verify_signature(&secret, &cookie.id, &cookie.sig)? {
        console_log!("read_session: session cookie signature invalid");
        return Ok(None);
    }

    let value = match kv(ctx)?.get(&cookie.id).text().await? {
        Some(value) => value,
        None => {
            console_log!("read_session: session id not found in KV");
            return Ok(None);
        }
    };

    let data: SessionData = match serde_json::from_str(&value) {
        Ok(data) => data,
        Err(_) => {
            console_log!("read_session: failed to parse session data from KV");
            return Ok(None);
        }
    };

    let account = fetch_account_by_provider(ctx, &data.provider, &data.provider_id).await?;
    if account.is_none() {
        console_log!("read_session: no account found for session");
    }
    Ok(account)
}

/// Requires a valid session and returns the authenticated account. Returns 401
/// if the request is not authenticated.
pub async fn require_account(req: &Request, ctx: &RouteContext<()>) -> Result<Account> {
    match read_session(req, ctx).await? {
        Some(account) => Ok(account),
        None => Err(worker::Error::RustError("Unauthorized".into())),
    }
}

/// Returns the currently authenticated account wrapped in `{ "account": ... }`,
/// or 401 if the request is not authenticated.
pub async fn me(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match read_session(&req, &ctx).await? {
        Some(account) => Response::from_json(&serde_json::json!({ "account": account })),
        None => Response::error("Unauthorized", 401),
    }
}

/// Clears the session cookie and removes the session from KV, then redirects
/// the browser to a safe frontend location.
///
/// The redirect target is read from the `redirect_to` query parameter. If it
/// is missing or not a safe relative path or allowed origin, the request's
/// public origin is used as a fallback.
pub async fn logout(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let is_https = is_https_request(&req);
    let cookie_name = session_cookie_name(is_https);

    let redirect_to = req
        .url()
        .ok()
        .and_then(|url| safe_redirect_target(is_https, &url, "redirect_to"))
        .unwrap_or_else(|| public_origin(&req));

    let cookie_header = match req.headers().get("Cookie")? {
        Some(value) => value,
        None => {
            return build_logout_response(redirect_to, cookie_name, is_https);
        }
    };

    let encoded = cookie_header
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.strip_prefix(cookie_name))
        .filter_map(|rest| rest.strip_prefix('='))
        .next();

    if let Some(encoded) = encoded
        && let Ok(decoded) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded)
        && let Ok(cookie) = serde_json::from_slice::<SessionCookie>(&decoded)
    {
        let secret = session_secret(&ctx)?;
        if verify_signature(&secret, &cookie.id, &cookie.sig)? {
            let _ = kv(&ctx)?.delete(&cookie.id).await;
        }
    }

    build_logout_response(redirect_to, cookie_name, is_https)
}

fn build_logout_response(origin: String, cookie_name: &str, is_https: bool) -> Result<Response> {
    let headers = Headers::new();
    headers.set("Set-Cookie", &build_clear_cookie(cookie_name, is_https))?;
    headers.set("Location", &origin)?;

    Ok(ResponseBuilder::new()
        .with_status(302)
        .with_headers(headers)
        .empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn https_origin_detected() {
        assert!(is_https_origin("https://cadiotheka.com"));
        assert!(!is_https_origin("http://localhost:8080"));
        assert!(!is_https_origin("http://localhost:8787"));
    }

    #[test]
    fn session_cookie_name_matches_scheme() {
        assert_eq!(session_cookie_name(true), SESSION_COOKIE_NAME_PREFIX);
        assert_eq!(session_cookie_name(false), SESSION_COOKIE_NAME_PLAIN);
    }

    #[test]
    fn session_cookie_is_secure_and_cross_site_on_https() {
        let cookie = build_session_cookie("__Host-session", "abc", true);
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=None"));
        assert!(cookie.contains("__Host-session"));
    }

    #[test]
    fn session_cookie_is_not_secure_on_http() {
        let cookie = build_session_cookie("session", "abc", false);
        assert!(!cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Lax"));
    }
}
