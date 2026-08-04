use serde::Deserialize;
use worker::{
    Env, Fetch, Headers, Method, Request, RequestInit, Response, Result, console_log, wasm_bindgen,
};

use crate::utils::{error_response, rust_err};

/// Endpoint for Cloudflare Turnstile server-side verification.
const TURNSTILE_VERIFY_URL: &str = "https://challenges.cloudflare.com/turnstile/v0/siteverify";

/// Environment secret name holding the Turnstile widget secret.
const TURNSTILE_SECRET_KEY: &str = "TURNSTILE_SECRET";

/// Parsed siteverify response from Cloudflare.
#[derive(Debug, Deserialize)]
struct SiteVerifyResponse {
    success: bool,
    #[serde(default, rename = "error-codes")]
    error_codes: Vec<String>,
}

/// Verifies a Turnstile token from the request.
///
/// Reads the token from either the `X-Turnstile-Token` header or a JSON body
/// field named `cf-turnstile-response`. If the token is missing or siteverify
/// returns `success: false`, this returns a 403 response. Otherwise it returns
/// `Ok(None)` so the caller can continue processing.
///
/// This helper should be called early in any handler that needs bot
/// protection.
pub async fn verify_turnstile_token(
    req: &mut Request,
    ctx: &worker::RouteContext<()>,
) -> Result<Option<Response>> {
    let token = extract_token(req).await;

    let Some(token) = token else {
        return Ok(Some(error_response("Turnstile token required", 403)?));
    };

    if token.trim().is_empty() {
        return Ok(Some(error_response("Turnstile token required", 403)?));
    }

    match verify_with_cloudflare(&ctx.env, &token, &client_ip(req)).await {
        Ok((true, _)) => Ok(None),
        Ok((false, error_codes)) => {
            console_log!("Turnstile verification failed: {error_codes:?}");
            Ok(Some(error_response("Turnstile verification failed", 403)?))
        }
        Err(err) => {
            console_log!("Turnstile siteverify error: {err}");
            Ok(Some(error_response("Turnstile verification failed", 403)?))
        }
    }
}

/// Reads the Turnstile token from the request, preferring the header.
async fn extract_token(req: &mut Request) -> Option<String> {
    if let Ok(Some(header)) = req.headers().get("X-Turnstile-Token") {
        return Some(header);
    }

    let Ok(body_text) = req.text().await else {
        return None;
    };

    let parsed: serde_json::Value = serde_json::from_str(&body_text).unwrap_or_default();
    parsed
        .get("cf-turnstile-response")
        .or_else(|| parsed.get("_turnstile_token"))
        .and_then(|value| value.as_str())
        .map(String::from)
}

/// Calls Cloudflare's Turnstile siteverify endpoint.
async fn verify_with_cloudflare(
    env: &Env,
    token: &str,
    remoteip: &str,
) -> Result<(bool, Vec<String>)> {
    let secret = env
        .secret(TURNSTILE_SECRET_KEY)
        .map_err(|err| rust_err(format!("missing TURNSTILE_SECRET: {err}")))?
        .to_string();

    let body = serde_urlencoded::to_string([
        ("secret", secret.as_str()),
        ("response", token),
        ("remoteip", remoteip),
    ])
    .map_err(rust_err)?;

    let headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(wasm_bindgen::JsValue::from_str(&body)));

    let request = Request::new_with_init(TURNSTILE_VERIFY_URL, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    if response.status_code() != 200 {
        let text = response.text().await.unwrap_or_default();
        return Err(rust_err(format!(
            "siteverify returned HTTP {}: {text}",
            response.status_code()
        )));
    }

    let text = response.text().await?;
    let parsed: SiteVerifyResponse = serde_json::from_str(&text).map_err(|err| {
        rust_err(format!(
            "failed to parse siteverify response: {err} (body={text})"
        ))
    })?;

    Ok((parsed.success, parsed.error_codes))
}

/// Best-effort client IP from Cloudflare/forwarded headers.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_url_is_canonical() {
        assert_eq!(
            TURNSTILE_VERIFY_URL,
            "https://challenges.cloudflare.com/turnstile/v0/siteverify"
        );
    }

    #[test]
    fn secret_key_name_is_turnstile_secret() {
        assert_eq!(TURNSTILE_SECRET_KEY, "TURNSTILE_SECRET");
    }
}
