use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, EmptyExtraTokenFields, PkceCodeChallenge,
    RedirectUrl, Scope, StandardTokenResponse, TokenResponse,
    basic::{BasicClient, BasicTokenType},
};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use worker::*;

use crate::api::accounts::{
    Account, OAuthProfile, create_oauth_account, fetch_account, fetch_account_by_provider,
    link_oauth_account,
};
use crate::api::session::{create_session, read_session};
use crate::utils::{
    check_rate_limit, error_response, is_https_request, public_origin, query_param, rust_err,
    safe_redirect_target,
};

const AUTH_KV_BINDING: &str = "AUTH";
const OAUTH_STATE_TTL_SECONDS: u64 = 10 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Provider {
    GitHub,
    Google,
}

impl Provider {
    fn as_str(&self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
        }
    }

    fn credentials(&self) -> (&'static str, &'static str) {
        match self {
            Self::GitHub => ("GITHUB_CLIENT_ID", "GITHUB_CLIENT_SECRET"),
            Self::Google => ("GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET"),
        }
    }

    fn auth_url(&self) -> &'static str {
        match self {
            Self::GitHub => "https://github.com/login/oauth/authorize",
            Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    fn token_url(&self) -> &'static str {
        match self {
            Self::GitHub => "https://github.com/login/oauth/access_token",
            Self::Google => "https://oauth2.googleapis.com/token",
        }
    }

    fn scopes(&self) -> &'static [&'static str] {
        match self {
            Self::GitHub => &["read:user", "user:email"],
            Self::Google => &["openid", "email", "profile"],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthState {
    provider: Provider,
    pkce_verifier: String,
    redirect_to: String,
    link_account_id: Option<String>,
}

fn oauth_client(ctx: &RouteContext<()>, provider: Provider) -> Result<BasicClient> {
    let (id_key, secret_key) = provider.credentials();

    let client_id = ClientId::new(ctx.env.secret(id_key)?.to_string());
    let client_secret = ClientSecret::new(ctx.env.secret(secret_key)?.to_string());

    Ok(BasicClient::new(client_id).set_client_secret(client_secret))
}

pub async fn github_login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "oauth_login").await? {
        return Ok(response);
    }
    let redirect_to = req
        .url()
        .ok()
        .and_then(|url| safe_redirect_target(is_https_request(&req), &url, "redirect_to"))
        .unwrap_or_else(|| "/".to_string());
    let link_account_id = read_session(&req, &ctx).await?.map(|a| a.id);
    let url = login_url(req, ctx, Provider::GitHub, &redirect_to, link_account_id).await?;
    Response::from_json(&serde_json::json!({ "url": url }))
}

pub async fn google_login(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "oauth_login").await? {
        return Ok(response);
    }
    let redirect_to = req
        .url()
        .ok()
        .and_then(|url| safe_redirect_target(is_https_request(&req), &url, "redirect_to"))
        .unwrap_or_else(|| "/".to_string());
    let link_account_id = read_session(&req, &ctx).await?.map(|a| a.id);
    let url = login_url(req, ctx, Provider::Google, &redirect_to, link_account_id).await?;
    Response::from_json(&serde_json::json!({ "url": url }))
}

async fn login_url(
    req: Request,
    ctx: RouteContext<()>,
    provider: Provider,
    redirect_to: &str,
    link_account_id: Option<String>,
) -> Result<String> {
    let client = oauth_client(&ctx, provider)?
        .set_auth_uri(AuthUrl::new(provider.auth_url().to_string()).map_err(rust_err)?);

    let redirect_uri = RedirectUrl::new(format!(
        "{}{}",
        public_origin(&req),
        match provider {
            Provider::GitHub => crate::routes::AUTH_GITHUB_CALLBACK,
            Provider::Google => crate::routes::AUTH_GOOGLE_CALLBACK,
        }
    ))
    .map_err(rust_err)?;

    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();

    let (url, csrf_token) = client
        .authorize_url(CsrfToken::new_random)
        .set_redirect_uri(Cow::Owned(redirect_uri))
        .set_pkce_challenge(challenge)
        .add_scopes(
            provider
                .scopes()
                .iter()
                .map(|scope| Scope::new((*scope).to_string())),
        )
        .url();

    let state = OAuthState {
        provider,
        pkce_verifier: verifier.secret().clone(),
        redirect_to: redirect_to.to_string(),
        link_account_id,
    };

    kv(&ctx)?
        .put(
            &format!("oauth_state:{}", csrf_token.secret()),
            &serde_json::to_string(&state).map_err(rust_err)?,
        )?
        .expiration_ttl(OAUTH_STATE_TTL_SECONDS)
        .execute()
        .await?;

    Ok(url.to_string())
}

pub async fn github_callback(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    callback(req, ctx, Provider::GitHub).await
}

pub async fn google_callback(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    callback(req, ctx, Provider::Google).await
}

async fn callback(req: Request, ctx: RouteContext<()>, provider: Provider) -> Result<Response> {
    if let Some(response) = check_rate_limit(&req, &ctx, "oauth_callback").await? {
        return Ok(response);
    }
    let url = req.url()?;

    let code = query_param(&url, "code").ok_or_else(|| rust_err("missing oauth code"))?;

    let state = query_param(&url, "state").ok_or_else(|| rust_err("missing oauth state"))?;

    let key = format!("oauth_state:{state}");

    let value = kv(&ctx)?
        .get(&key)
        .text()
        .await?
        .ok_or_else(|| rust_err("invalid oauth state"))?;

    let state: OAuthState = serde_json::from_str(&value).map_err(rust_err)?;

    if state.provider != provider {
        return error_response("invalid provider", 400);
    }

    kv(&ctx)?.delete(&key).await?;

    let token = exchange_code(&ctx, provider, code, state.pkce_verifier, &req).await?;

    let account = if let Some(account_id) = state.link_account_id {
        let (provider_id, _profile) = fetch_provider_profile(&ctx, provider, &token).await?;
        link_oauth_account(&ctx, &account_id, provider.as_str(), &provider_id).await?;
        fetch_account(&ctx, &account_id)
            .await?
            .ok_or_else(|| rust_err("account to link not found"))?
    } else {
        fetch_or_create_account(&ctx, provider, &token).await?
    };

    let cookie = create_session(&ctx, &account, &req).await?;

    let redirect_to = html_escape(&state.redirect_to);
    let html = format!(
        r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <meta http-equiv='refresh' content='0; url={redirect_to}'>
    <title>Redirecting...</title>
  </head>
  <body>
    <p>Redirecting to <a href='{redirect_to}'>Cadiotheka</a>...</p>
    <script>window.location.replace('{redirect_to}');</script>
  </body>
</html>"#
    );

    let headers = Headers::new();
    headers.set("Content-Type", "text/html; charset=utf-8")?;
    headers.set("Set-Cookie", &cookie)?;

    Ok(ResponseBuilder::new()
        .with_status(200)
        .with_headers(headers)
        .body(ResponseBody::Body(html.into())))
}

/// Minimal HTML escaping for single-quoted attributes.
fn html_escape(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            '&' => "&amp;".to_string(),
            '\'' => "&#39;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            _ => ch.to_string(),
        })
        .collect()
}

async fn exchange_code(
    ctx: &RouteContext<()>,
    provider: Provider,
    code: String,
    verifier: String,
    req: &Request,
) -> Result<String> {
    let client_id = ctx.env.secret(provider.credentials().0)?.to_string();
    let client_secret = ctx.env.secret(provider.credentials().1)?.to_string();

    let redirect_uri = format!(
        "{}{}",
        public_origin(req),
        match provider {
            Provider::GitHub => crate::routes::AUTH_GITHUB_CALLBACK,
            Provider::Google => crate::routes::AUTH_GOOGLE_CALLBACK,
        }
    );

    let body = serde_urlencoded::to_string([
        ("grant_type", "authorization_code"),
        ("code", &code),
        ("redirect_uri", &redirect_uri),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("code_verifier", &verifier),
    ])
    .map_err(rust_err)?;

    let headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;
    headers.set("Accept", "application/json")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_headers(headers)
        .with_body(Some(wasm_bindgen::JsValue::from_str(&body)));

    let request = Request::new_with_init(provider.token_url(), &init)?;

    let mut response = Fetch::Request(request).send().await?;

    let text = response.text().await?;

    if response.status_code() != 200 {
        return Err(rust_err(format!(
            "token exchange failed (HTTP {}): {text}",
            response.status_code()
        )));
    }

    if text.trim().is_empty() {
        return Err(rust_err("token exchange returned an empty response"));
    }

    let token: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType> =
        serde_json::from_str(&text).map_err(|e| {
            rust_err(format!(
                "token exchange returned invalid JSON ({}): {e}",
                text.chars().take(200).collect::<String>()
            ))
        })?;

    Ok(token.access_token().secret().clone())
}

async fn fetch_or_create_account(
    ctx: &RouteContext<()>,
    provider: Provider,
    access_token: &str,
) -> Result<Account> {
    let (provider_id, profile) = fetch_provider_profile(ctx, provider, access_token).await?;

    if let Some(account) = fetch_account_by_provider(ctx, provider.as_str(), &provider_id).await? {
        return Ok(account);
    }

    create_oauth_account(ctx, provider.as_str(), &provider_id, profile).await
}

async fn fetch_provider_profile(
    ctx: &RouteContext<()>,
    provider: Provider,
    access_token: &str,
) -> Result<(String, OAuthProfile)> {
    match provider {
        Provider::GitHub => fetch_github_profile(ctx, access_token).await,
        Provider::Google => fetch_google_profile(ctx, access_token).await,
    }
}

async fn fetch_github_profile(
    _ctx: &RouteContext<()>,
    access_token: &str,
) -> Result<(String, OAuthProfile)> {
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {access_token}"))?;
    headers.set("Accept", "application/vnd.github+json")?;
    headers.set("User-Agent", "cadiotheka")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers.clone());

    let user_req = Request::new_with_init("https://api.github.com/user", &init)?;
    let user: GitHubUser = fetch_json(user_req).await?;

    let mut email_init = RequestInit::new();
    email_init.with_method(Method::Get).with_headers(headers);

    let email_req = Request::new_with_init("https://api.github.com/user/emails", &email_init)?;
    let emails: Vec<GitHubEmail> = fetch_json(email_req).await?;
    let email = emails
        .iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email.clone())
        .or_else(|| user.email.clone())
        .unwrap_or_default();

    let provider_id = user.id.to_string();
    let profile = OAuthProfile {
        preferred_username: user.login.clone(),
        display_name: user.name.unwrap_or_else(|| user.login.clone()),
        email,
        avatar_url: user.avatar_url,
        bio: user.bio.unwrap_or_default(),
    };

    Ok((provider_id, profile))
}

async fn fetch_google_profile(
    _ctx: &RouteContext<()>,
    access_token: &str,
) -> Result<(String, OAuthProfile)> {
    let headers = Headers::new();
    headers.set("Authorization", &format!("Bearer {access_token}"))?;

    let mut init = RequestInit::new();
    init.with_method(Method::Get).with_headers(headers);

    let user_req = Request::new_with_init("https://www.googleapis.com/oauth2/v3/userinfo", &init)?;
    let user: GoogleUser = fetch_json(user_req).await?;

    let provider_id = user.sub;
    let email = user.email.clone().unwrap_or_default();
    let preferred_username = email.split('@').next().unwrap_or("user");

    let profile = OAuthProfile {
        preferred_username: preferred_username.to_string(),
        display_name: user.name.unwrap_or_else(|| preferred_username.to_string()),
        email,
        avatar_url: user.picture,
        bio: String::new(),
    };

    Ok((provider_id, profile))
}

async fn fetch_json<T: serde::de::DeserializeOwned>(req: Request) -> Result<T> {
    let mut response = Fetch::Request(req).send().await?;
    let text = response.text().await?;
    if response.status_code() != 200 {
        return Err(rust_err(format!("provider api request failed: {text}")));
    }
    serde_json::from_str(&text).map_err(rust_err)
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: u64,
    login: String,
    name: Option<String>,
    email: Option<String>,
    avatar_url: Option<String>,
    bio: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

#[derive(Debug, Deserialize)]
struct GoogleUser {
    sub: String,
    name: Option<String>,
    email: Option<String>,
    picture: Option<String>,
}

fn kv(ctx: &RouteContext<()>) -> Result<KvStore> {
    ctx.env.kv(AUTH_KV_BINDING)
}
