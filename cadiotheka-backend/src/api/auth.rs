use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use worker::*;

use crate::DB_BINDING;
use crate::api::accounts::{Account, create_oauth_account, fetch_account_by_provider};

/// Name of the KV namespace used for OAuth state and sessions.
const AUTH_KV_BINDING: &str = "AUTH_KV";

/// Name of the secret used to sign session cookies.
const SESSION_SECRET_BINDING: &str = "SESSION_SECRET";

/// Name of the HTTP-only session cookie.
const SESSION_COOKIE_NAME: &str = "session";

/// Session lifetime in seconds (7 days).
const SESSION_TTL_SECONDS: u64 = 7 * 24 * 60 * 60;

/// OAuth state lifetime in seconds (10 minutes).
const OAUTH_STATE_TTL_SECONDS: u64 = 10 * 60;

/// OAuth providers supported by this worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Provider {
    GitHub,
    Google,
}

impl Provider {
    fn as_str(&self) -> &'static str {
        match self {
            Provider::GitHub => "github",
            Provider::Google => "google",
        }
    }

    fn client_id_secret_binding(&self) -> (&'static str, &'static str) {
        match self {
            Provider::GitHub => ("GITHUB_CLIENT_ID", "GITHUB_CLIENT_SECRET"),
            Provider::Google => ("GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET"),
        }
    }

    fn authorize_url(&self) -> &'static str {
        match self {
            Provider::GitHub => "https://github.com/login/oauth/authorize",
            Provider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    fn token_url(&self) -> &'static str {
        match self {
            Provider::GitHub => "https://github.com/login/oauth/access_token",
            Provider::Google => "https://oauth2.googleapis.com/token",
        }
    }

    fn scopes(&self) -> &'static str {
        match self {
            Provider::GitHub => "read:user user:email",
            Provider::Google => "openid email profile",
        }
    }
}

impl std::str::FromStr for Provider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(Provider::GitHub),
            "google" => Ok(Provider::Google),
            _ => Err(()),
        }
    }
}

/// Data stored in KV for an OAuth state parameter.
#[derive(Serialize, Deserialize, Debug)]
struct OAuthState {
    code_verifier: String,
    provider: String,
}

/// Data stored in KV for a session.
#[derive(Serialize, Deserialize, Debug)]
struct SessionData {
    account_id: String,
}

/// Public JSON returned by `/auth/me`.
#[derive(Serialize, Debug)]
struct MeResponse {
    account: Account,
}

/// Error JSON returned for auth failures.
#[derive(Serialize, Debug)]
struct ErrorResponse {
    error: String,
}

/// Redirects the browser to the GitHub OAuth authorization endpoint.
pub async fn github_login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    login_redirect(req, ctx, Provider::GitHub).await
}

/// Redirects the browser to the Google OAuth authorization endpoint.
pub async fn google_login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    login_redirect(req, ctx, Provider::Google).await
}

/// Builds the OAuth authorization redirect for a provider.
async fn login_redirect(
    req: Request,
    ctx: RouteContext<()>,
    provider: Provider,
) -> Result<Response> {
    let (client_id, _) = client_credentials(&ctx, provider)?;
    let origin = public_origin(&req);
    let redirect_uri = format!("{origin}/auth/{}/callback", provider.as_str());

    let code_verifier = generate_code_verifier();
    let code_challenge = sha256_base64url(&code_verifier);
    let state = generate_state();

    let state_data = OAuthState {
        code_verifier,
        provider: provider.as_str().to_string(),
    };
    kv(&ctx)?
        .put(
            &format!("oauth_state:{state}"),
            &serde_json::to_string(&state_data).map_err(rust_err)?,
        )?
        .expiration_ttl(OAUTH_STATE_TTL_SECONDS)
        .execute()
        .await?;

    let mut authorize_url = url::Url::parse(provider.authorize_url())
        .map_err(|e| rust_err(format!("failed to parse authorize url: {e}")))?;
    {
        let mut query = authorize_url.query_pairs_mut();
        query.append_pair("client_id", &client_id);
        query.append_pair("redirect_uri", &redirect_uri);
        query.append_pair("response_type", "code");
        query.append_pair("scope", provider.scopes());
        query.append_pair("state", &state);
        query.append_pair("code_challenge", &code_challenge);
        query.append_pair("code_challenge_method", "S256");
    }

    Response::redirect(authorize_url)
}

/// Handles the OAuth callback from GitHub.
pub async fn github_callback(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    callback(req, ctx, Provider::GitHub).await
}

/// Handles the OAuth callback from Google.
pub async fn google_callback(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    callback(req, ctx, Provider::Google).await
}

/// Exchanges the authorization code for an access token, fetches user info, and
/// creates or updates the session.
async fn callback(req: Request, ctx: RouteContext<()>, provider: Provider) -> Result<Response> {
    let url = req.url()?;
    let code =
        query_param(&url, "code").ok_or_else(|| worker::Error::RustError("missing code".into()))?;
    let state = query_param(&url, "state")
        .ok_or_else(|| worker::Error::RustError("missing state".into()))?;

    let state_key = format!("oauth_state:{state}");
    let state_json = kv(&ctx)?
        .get(&state_key)
        .text()
        .await?
        .ok_or_else(|| worker::Error::RustError("invalid or expired state".into()))?;
    let state_data: OAuthState = serde_json::from_str(&state_json)
        .map_err(|e| rust_err(format!("failed to parse oauth state: {e}")))?;

    // Validate the returned provider matches the stored provider.
    if state_data.provider != provider.as_str() {
        return Response::error("invalid oauth state", 400);
    }

    // Delete the state immediately so it cannot be reused.
    kv(&ctx)?.delete(&state_key).await?;

    let token_response =
        exchange_code(&ctx, provider, &code, &state_data.code_verifier, &req).await?;
    let access_token = token_response
        .access_token
        .ok_or_else(|| worker::Error::RustError("missing access_token".into()))?;

    let user = fetch_user_info(provider, &access_token).await?;
    let account = upsert_account(&ctx, provider, &user).await?;
    let session_token = create_session(&ctx, &account.id).await?;
    let cookie = session_cookie(&ctx, &session_token, &req)?;

    let mut resp = Response::redirect(public_origin(&req).parse::<Url>()?)?;
    resp.headers_mut().set("Set-Cookie", &cookie)?;
    Ok(resp)
}

/// Returns the current authenticated account, or 401.
pub async fn me(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    match current_account(&req, &ctx).await? {
        Some(account) => Response::from_json(&MeResponse { account }),
        None => {
            let mut resp = Response::from_json(&ErrorResponse {
                error: "not authenticated".into(),
            })?;
            resp.headers_mut().set("WWW-Authenticate", "Cookie")?;
            Ok(resp.with_status(401))
        }
    }
}

/// Clears the session cookie and KV entry, then redirects home.
pub async fn logout(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some((token, _sig)) = session_from_request(&req) {
        let _ = kv(&ctx)?.delete(&format!("session:{token}")).await;
    }

    let cookie = clear_session_cookie(&req);
    let mut resp = Response::redirect(public_origin(&req).parse::<Url>()?)?;
    resp.headers_mut().set("Set-Cookie", &cookie)?;
    Ok(resp)
}

/// Fetches the account associated with the current request's session, if any.
async fn current_account(req: &Request, ctx: &RouteContext<()>) -> Result<Option<Account>> {
    let Some((token, _sig)) = session_from_request(req) else {
        return Ok(None);
    };

    let Some(session_json) = kv(ctx)?.get(&format!("session:{token}")).text().await? else {
        return Ok(None);
    };
    let session: SessionData = serde_json::from_str(&session_json)
        .map_err(|e| rust_err(format!("failed to parse session: {e}")))?;

    let result = ctx.env.d1(DB_BINDING)?.prepare(
        "SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified, provider, provider_id FROM accounts WHERE id = ?1",
    )
    .bind(&[session.account_id.into()])?
    .all()
    .await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    Ok(accounts.pop())
}

/// Middleware-style helper that returns 401 if the request is not authenticated.
#[allow(dead_code)]
pub async fn require_account(req: &Request, ctx: &RouteContext<()>) -> Result<Account> {
    current_account(req, ctx)
        .await?
        .ok_or_else(|| worker::Error::RustError("unauthenticated".into()))
}

/// Exchanges an authorization code for an access token using PKCE.
async fn exchange_code(
    ctx: &RouteContext<()>,
    provider: Provider,
    code: &str,
    code_verifier: &str,
    req: &Request,
) -> Result<TokenResponse> {
    let (client_id, client_secret) = client_credentials(ctx, provider)?;
    let origin = public_origin(req);
    let redirect_uri = format!("{origin}/auth/{}/callback", provider.as_str());

    let body = serde_urlencoded::to_string(&TokenRequest {
        client_id: &client_id,
        client_secret: &client_secret,
        code: code.to_string(),
        redirect_uri: &redirect_uri,
        grant_type: "authorization_code",
        code_verifier: code_verifier.to_string(),
    })
    .map_err(|e| rust_err(format!("failed to encode token request: {e}")))?;

    let token_url = url::Url::parse(provider.token_url())
        .map_err(|e| rust_err(format!("failed to parse token url: {e}")))?;

    let headers = Headers::new();
    headers.set("Accept", "application/json")?;
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(wasm_bindgen::JsValue::from_str(&body)));

    let token_req = Request::new_with_init(token_url.as_ref(), &init)?;

    let mut resp = Fetch::Request(token_req).send().await?;
    let status = resp.status_code();
    let body_text = resp.text().await?;
    if status != 200 {
        return Err(rust_err(format!(
            "token exchange failed: HTTP {status}: {body_text}"
        )));
    }
    serde_json::from_str(&body_text)
        .map_err(|e| rust_err(format!("failed to parse token response: {e}: {body_text}")))
}

/// Fetches normalized user information from a provider's userinfo endpoint.
async fn fetch_user_info(provider: Provider, access_token: &str) -> Result<OAuthUser> {
    match provider {
        Provider::GitHub => fetch_github_user(access_token).await,
        Provider::Google => fetch_google_user(access_token).await,
    }
}

/// Fetches a GitHub user and their primary email address.
async fn fetch_github_user(access_token: &str) -> Result<OAuthUser> {
    let user = github_api::<GitHubUser>("https://api.github.com/user", access_token).await?;
    let email = if user.email.as_deref().unwrap_or("").is_empty() {
        let emails =
            github_api::<Vec<GitHubEmail>>("https://api.github.com/user/emails", access_token)
                .await?;
        emails
            .iter()
            .find(|e| e.primary && e.verified)
            .or_else(|| emails.iter().find(|e| e.verified))
            .or_else(|| emails.first())
            .map(|e| e.email.clone())
            .unwrap_or_default()
    } else {
        user.email.unwrap_or_default()
    };

    Ok(OAuthUser {
        provider_id: user.id.to_string(),
        preferred_username: user.login,
        display_name: user.name.unwrap_or_default(),
        email,
        avatar_url: user.avatar_url,
    })
}

/// Makes a GitHub API GET request and parses the JSON response.
async fn github_api<T: serde::de::DeserializeOwned>(url: &str, access_token: &str) -> Result<T> {
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {access_token}"))?;
    headers.set("Accept", "application/vnd.github+json")?;
    headers.set("User-Agent", "cadiotheka")?;
    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);
    let req = Request::new_with_init(url, &init)?;

    let mut resp = Fetch::Request(req).send().await?;
    let status = resp.status_code();
    let body_text = resp.text().await?;
    if status != 200 {
        return Err(rust_err(format!(
            "github api error: HTTP {status}: {body_text}"
        )));
    }
    serde_json::from_str(&body_text)
        .map_err(|e| rust_err(format!("failed to parse github response: {e}: {body_text}")))
}

/// Fetches a Google user.
async fn fetch_google_user(access_token: &str) -> Result<OAuthUser> {
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {access_token}"))?;
    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);
    let req = Request::new_with_init("https://www.googleapis.com/oauth2/v2/userinfo", &init)?;

    let mut resp = Fetch::Request(req).send().await?;
    let status = resp.status_code();
    let body_text = resp.text().await?;
    if status != 200 {
        return Err(rust_err(format!(
            "google api error: HTTP {status}: {body_text}"
        )));
    }
    let user: GoogleUser = serde_json::from_str(&body_text)
        .map_err(|e| rust_err(format!("failed to parse google response: {e}: {body_text}")))?;

    Ok(OAuthUser {
        provider_id: user.id,
        preferred_username: user.email.split('@').next().unwrap_or("user").to_string(),
        display_name: user.name,
        email: user.email,
        avatar_url: user.picture,
    })
}

/// Creates an account if it does not exist, otherwise returns the existing one.
async fn upsert_account(
    ctx: &RouteContext<()>,
    provider: Provider,
    user: &OAuthUser,
) -> Result<Account> {
    if let Some(account) =
        fetch_account_by_provider(ctx, provider.as_str(), &user.provider_id).await?
    {
        return Ok(account);
    }

    let display_name = if user.display_name.is_empty() {
        user.preferred_username.clone()
    } else {
        user.display_name.clone()
    };

    create_oauth_account(
        ctx,
        provider.as_str(),
        &user.provider_id,
        &user.preferred_username,
        &display_name,
        &user.email,
        user.avatar_url.clone(),
    )
    .await
}

/// Creates a session in KV and returns the session token.
async fn create_session(ctx: &RouteContext<()>, account_id: &str) -> Result<String> {
    let mut token_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut token_bytes);
    let token = hex::encode(token_bytes);

    let session = SessionData {
        account_id: account_id.to_string(),
    };
    kv(ctx)?
        .put(
            &format!("session:{token}"),
            &serde_json::to_string(&session).map_err(rust_err)?,
        )?
        .expiration_ttl(SESSION_TTL_SECONDS)
        .execute()
        .await?;

    Ok(token)
}

/// Builds the signed `Set-Cookie` header for a session token.
fn session_cookie(ctx: &RouteContext<()>, token: &str, req: &Request) -> Result<String> {
    let signature = sign_cookie_value(ctx, token)?;
    let value = format!("{token}.{signature}");
    let secure = !is_localhost(req);
    let mut cookie = format!(
        "{SESSION_COOKIE_NAME}={value}; HttpOnly; SameSite=Lax; Path=/; Max-Age={SESSION_TTL_SECONDS}"
    );
    if secure {
        cookie.push_str("; Secure");
    }
    Ok(cookie)
}

/// Builds a `Set-Cookie` header that clears the session.
fn clear_session_cookie(req: &Request) -> String {
    let mut cookie = format!("{SESSION_COOKIE_NAME}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0");
    if !is_localhost(req) {
        cookie.push_str("; Secure");
    }
    cookie
}

/// Signs a session token using HMAC-SHA256 with the configured secret.
fn sign_cookie_value(ctx: &RouteContext<()>, token: &str) -> Result<String> {
    let secret = ctx.env.secret(SESSION_SECRET_BINDING)?.to_string();
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes())
        .map_err(|e| rust_err(format!("failed to initialize hmac: {e}")))?;
    mac.update(token.as_bytes());
    let result = mac.finalize();
    Ok(hex::encode(result.into_bytes()))
}

/// Verifies the signature portion of a session cookie value.
#[allow(dead_code)]
fn verify_cookie_value(ctx: &RouteContext<()>, token: &str, signature: &str) -> Result<bool> {
    let expected = sign_cookie_value(ctx, token)?;
    Ok(expected.eq_ignore_ascii_case(signature))
}

/// Extracts the session token from the request's Cookie header.
fn session_from_request(req: &Request) -> Option<(String, String)> {
    let headers = req.headers();
    let cookie_header = headers.get("Cookie").ok()??;
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(&format!("{SESSION_COOKIE_NAME}=")) {
            let mut iter = value.splitn(2, '.');
            let token = iter.next()?.to_string();
            let signature = iter.next()?.to_string();
            return Some((token, signature));
        }
    }
    None
}

/// Reads a query parameter from a URL.
fn query_param(url: &url::Url, name: &str) -> Option<String> {
    url.query_pairs().find_map(|(k, v)| {
        if k == name {
            Some(v.into_owned())
        } else {
            None
        }
    })
}

/// Returns the public origin of the request, used for OAuth redirect URIs.
fn public_origin(req: &Request) -> String {
    let url = req
        .url()
        .unwrap_or_else(|_| url::Url::parse("https://cadiotheka.com").unwrap());
    let scheme = url.scheme();
    let host = url.host_str().unwrap_or("cadiotheka.com");
    if let Some(port) = url.port() {
        format!("{scheme}://{host}:{port}")
    } else {
        format!("{scheme}://{host}")
    }
}

/// Whether the request is served on localhost (development).
fn is_localhost(req: &Request) -> bool {
    req.url()
        .ok()
        .and_then(|u| u.host_str().map(|h| h == "localhost" || h == "127.0.0.1"))
        .unwrap_or(false)
}

/// Loads client credentials from Wrangler secrets.
fn client_credentials(ctx: &RouteContext<()>, provider: Provider) -> Result<(String, String)> {
    let (id_binding, secret_binding) = provider.client_id_secret_binding();
    let client_id = ctx.env.secret(id_binding)?.to_string();
    let client_secret = ctx.env.secret(secret_binding)?.to_string();
    Ok((client_id, client_secret))
}

/// Returns the configured KV namespace for auth data.
fn kv(ctx: &RouteContext<()>) -> Result<KvStore> {
    ctx.env.kv(AUTH_KV_BINDING)
}

/// Generates a PKCE code verifier (43-128 URL-safe characters).
fn generate_code_verifier() -> String {
    let mut bytes = [0u8; 64];
    rand::thread_rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Returns the base64url-encoded SHA-256 hash of a string.
fn sha256_base64url(input: &str) -> String {
    use sha2::Digest;
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    URL_SAFE_NO_PAD.encode(hasher.finalize())
}

/// Generates an opaque OAuth state parameter.
fn generate_state() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    hex::encode(bytes)
}

/// Helper to create a `worker::Error::RustError` with a descriptive message.
fn rust_err<E: std::fmt::Display>(err: E) -> worker::Error {
    worker::Error::RustError(err.to_string())
}

/// Normalized user info returned by an OAuth provider.
#[derive(Debug)]
struct OAuthUser {
    provider_id: String,
    preferred_username: String,
    display_name: String,
    email: String,
    avatar_url: Option<String>,
}

/// Token request body sent to the OAuth token endpoint.
#[derive(Serialize)]
struct TokenRequest<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    code: String,
    redirect_uri: &'a str,
    grant_type: &'a str,
    code_verifier: String,
}

/// Token response from the OAuth token endpoint.
#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: Option<String>,
    #[allow(dead_code)]
    token_type: Option<String>,
}

/// GitHub user response.
#[derive(Deserialize, Debug)]
struct GitHubUser {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
}

/// GitHub email response.
#[derive(Clone, Deserialize, Debug)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

/// Google userinfo response.
#[derive(Deserialize, Debug)]
struct GoogleUser {
    id: String,
    email: String,
    name: String,
    picture: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_verifier_has_expected_length() {
        let verifier = generate_code_verifier();
        assert!(verifier.len() >= 43);
        assert!(verifier.len() <= 128);
        assert!(
            verifier
                .chars()
                .all(|c| { c.is_alphanumeric() || c == '-' || c == '_' || c == '~' || c == '.' })
        );
    }

    #[test]
    fn sha256_base64url_is_url_safe() {
        let hash = sha256_base64url("hello world");
        assert!(
            hash.chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        );
        assert!(!hash.contains('='));
    }

    #[test]
    fn query_param_parses_url() {
        let url = url::Url::parse("https://example.com/callback?code=abc&state=def").unwrap();
        assert_eq!(query_param(&url, "code"), Some("abc".to_string()));
        assert_eq!(query_param(&url, "state"), Some("def".to_string()));
        assert_eq!(query_param(&url, "missing"), None);
    }
}
