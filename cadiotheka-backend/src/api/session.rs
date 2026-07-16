use base64::Engine as _;
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use worker::*;

use crate::api::accounts::{Account, fetch_account_by_provider};
use crate::utils::{public_origin, rust_err};

const AUTH_KV_BINDING: &str = "AUTH_KV";
const SESSION_COOKIE_NAME: &str = "__Host-session";
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

/// Creates a session for the given account and returns a `Set-Cookie` header
/// value that the caller can attach to a redirect response.
pub async fn create_session(ctx: &RouteContext<()>, account: &Account) -> Result<String> {
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
    Ok(format!(
        "{SESSION_COOKIE_NAME}={encoded}; Max-Age={SESSION_TTL_SECONDS}; Path=/; HttpOnly; Secure; SameSite=Lax"
    ))
}

/// Reads the session cookie from the request, validates its signature, looks it
/// up in KV, and returns the associated account. Returns `None` if there is
/// no session or it is invalid/expired.
pub async fn read_session(req: &Request, ctx: &RouteContext<()>) -> Result<Option<Account>> {
    let cookie_header = match req.headers().get("Cookie")? {
        Some(value) => value,
        None => return Ok(None),
    };

    let encoded = cookie_header
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.strip_prefix(SESSION_COOKIE_NAME))
        .filter_map(|rest| rest.strip_prefix('='))
        .next();

    let encoded = match encoded {
        Some(value) => value,
        None => return Ok(None),
    };

    let decoded = match base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(None),
    };

    let cookie: SessionCookie = match serde_json::from_slice(&decoded) {
        Ok(cookie) => cookie,
        Err(_) => return Ok(None),
    };

    let secret = session_secret(ctx)?;
    if !verify_signature(&secret, &cookie.id, &cookie.sig)? {
        return Ok(None);
    }

    let value = match kv(ctx)?.get(&cookie.id).text().await? {
        Some(value) => value,
        None => return Ok(None),
    };

    let data: SessionData = match serde_json::from_str(&value) {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };

    let account = fetch_account_by_provider(ctx, &data.provider, &data.provider_id).await?;
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

/// Returns the currently authenticated account as JSON, or 401.
pub async fn me(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match read_session(&req, &ctx).await? {
        Some(account) => Response::from_json(&account),
        None => Response::error("Unauthorized", 401),
    }
}

/// Clears the session cookie and removes the session from KV.
pub async fn logout(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let cookie_header = match req.headers().get("Cookie")? {
        Some(value) => value,
        None => {
            return build_logout_response(public_origin(&req));
        }
    };

    let encoded = cookie_header
        .split(';')
        .map(str::trim)
        .filter_map(|part| part.strip_prefix(SESSION_COOKIE_NAME))
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

    build_logout_response(public_origin(&req))
}

fn build_logout_response(origin: String) -> Result<Response> {
    let headers = Headers::new();
    headers.set(
        "Set-Cookie",
        &format!("{SESSION_COOKIE_NAME}=; Max-Age=0; Path=/; HttpOnly; Secure; SameSite=Lax"),
    )?;
    headers.set("Location", &origin)?;

    Ok(ResponseBuilder::new()
        .with_status(302)
        .with_headers(headers)
        .empty())
}
