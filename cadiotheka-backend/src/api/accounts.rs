use serde::{Deserialize, Serialize};
use worker::*;

use crate::DB_BINDING;
use crate::api::session::require_account;
use crate::utils::{js_option, now_utc};

const SELECT_ACCOUNT_COLUMNS: &str = "SELECT a.id, a.username, a.display_name, a.email, a.role, a.bio, a.avatar_url, a.created_at, a.verified FROM accounts a";

/// A Cadiotheka account stored in D1.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
    /// D1 stores booleans as integers, so this field is an `i32` instead of a `bool`.
    pub verified: i32,
}

/// Payload used to create or update an account.
#[derive(Deserialize, Debug)]
pub struct AccountPayload {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub verified: i32,
}

/// Returns the D1 database binding configured for this worker.
fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1(DB_BINDING)
}

/// Fetches a single account by id, returning `None` when no row matches.
pub async fn fetch_account(ctx: &RouteContext<()>, id: &str) -> Result<Option<Account>> {
    let result = db(ctx)?
        .prepare(format!("{SELECT_ACCOUNT_COLUMNS} WHERE a.id = ?1"))
        .bind(&[id.into()])?
        .all()
        .await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    Ok(accounts.pop())
}

/// Fetches a single account by its OAuth provider and provider id.
pub async fn fetch_account_by_provider(
    ctx: &RouteContext<()>,
    provider: &str,
    provider_id: &str,
) -> Result<Option<Account>> {
    let result = db(ctx)?
        .prepare(format!(
            "{SELECT_ACCOUNT_COLUMNS} JOIN account_providers ap ON a.id = ap.account_id WHERE ap.provider = ?1 AND ap.provider_id = ?2"
        ))
        .bind(&[provider.into(), provider_id.into()])?
        .all()
        .await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    Ok(accounts.pop())
}

/// Returns the list of provider names linked to the given account.
pub async fn fetch_linked_providers(
    ctx: &RouteContext<()>,
    account_id: &str,
) -> Result<Vec<String>> {
    #[derive(Deserialize)]
    struct Row {
        provider: String,
    }

    let result = db(ctx)?
        .prepare("SELECT provider FROM account_providers WHERE account_id = ?1 ORDER BY created_at")
        .bind(&[account_id.into()])?
        .all()
        .await?;
    let rows: Vec<Row> = result.results::<Row>()?;
    Ok(rows.into_iter().map(|r| r.provider).collect())
}

/// Links an OAuth provider identity to an existing account.
///
/// Returns an error if the provider identity is already linked to a different
/// account.
pub async fn link_oauth_account(
    ctx: &RouteContext<()>,
    account_id: &str,
    provider: &str,
    provider_id: &str,
) -> Result<()> {
    let existing = fetch_account_by_provider(ctx, provider, provider_id).await?;
    if let Some(account) = existing {
        if account.id != account_id {
            return Err(worker::Error::RustError(
                "provider already linked to another account".into(),
            ));
        }
        return Ok(());
    }

    let created_at = now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| worker::Error::RustError(format!("failed to format timestamp: {e}")))?;

    db(ctx)?
        .prepare(
            "INSERT INTO account_providers (account_id, provider, provider_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(&[
            account_id.into(),
            provider.into(),
            provider_id.into(),
            created_at.into(),
        ])?
        .run()
        .await?;

    Ok(())
}

/// Unlinks an OAuth provider from the given account.
///
/// The provider that was used to create the account cannot be unlinked if it is
/// the only remaining provider, otherwise the account would become unaccessible.
/// Returns an error when the provider is not linked or is the sole provider.
pub async fn unlink_oauth_account(
    ctx: &RouteContext<()>,
    account_id: &str,
    provider: &str,
) -> Result<()> {
    let linked = fetch_linked_providers(ctx, account_id).await?;
    let position = linked.iter().position(|p| p == provider);

    let Some(_position) = position else {
        return Err(worker::Error::RustError("provider not linked".into()));
    };

    if linked.len() == 1 {
        return Err(worker::Error::RustError(
            "cannot unlink the only sign-in provider".into(),
        ));
    }

    db(ctx)?
        .prepare("DELETE FROM account_providers WHERE account_id = ?1 AND provider = ?2")
        .bind(&[account_id.into(), provider.into()])?
        .run()
        .await?;

    let providers = fetch_linked_providers(ctx, account_id).await?;
    let mut providers_iter = providers.iter();
    let first_provider = providers_iter.next();
    if first_provider == Some(&provider.to_string())
        && let Some(new_primary) = providers_iter.next()
    {
        db(ctx)?
            .prepare("UPDATE accounts SET provider = ?1 WHERE id = ?2")
            .bind(&[new_primary.into(), account_id.into()])?
            .run()
            .await?;
    }

    Ok(())
}

/// Fetches a single account by username, returning `None` when no row matches.
async fn fetch_account_by_username(
    ctx: &RouteContext<()>,
    username: &str,
) -> Result<Option<Account>> {
    let result = db(ctx)?
        .prepare(format!("{SELECT_ACCOUNT_COLUMNS} WHERE a.username = ?1"))
        .bind(&[username.into()])?
        .all()
        .await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    Ok(accounts.pop())
}

/// Profile information collected from an OAuth provider when creating a new
/// account.
pub struct OAuthProfile {
    pub preferred_username: String,
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: String,
}

/// Inserts a new account from an OAuth login.
///
/// The username is made unique by appending a short random suffix if the
/// provider's preferred login is already taken.
pub async fn create_oauth_account(
    ctx: &RouteContext<()>,
    provider: &str,
    provider_id: &str,
    profile: OAuthProfile,
) -> Result<Account> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| worker::Error::RustError(format!("failed to format timestamp: {e}")))?;
    let username = unique_username(ctx, &profile.preferred_username).await?;

    let account = Account {
        id: id.clone(),
        username: username.clone(),
        display_name: profile.display_name,
        email: profile.email,
        role: "creator".to_string(),
        bio: profile.bio,
        avatar_url: profile.avatar_url,
        created_at: created_at.clone(),
        verified: 1,
    };

    db(ctx)?
        .prepare(
            "INSERT INTO accounts (id, username, display_name, email, role, bio, avatar_url, created_at, verified) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&[
            id.clone().into(),
            username.into(),
            account.display_name.clone().into(),
            account.email.clone().into(),
            account.role.clone().into(),
            account.bio.clone().into(),
            js_option(account.avatar_url.clone()),
            account.created_at.clone().into(),
            account.verified.into(),
        ])?
        .run()
        .await?;

    db(ctx)?
        .prepare(
            "INSERT INTO account_providers (account_id, provider, provider_id, created_at) VALUES (?1, ?2, ?3, ?4)",
        )
        .bind(&[
            id.into(),
            provider.into(),
            provider_id.into(),
            created_at.into(),
        ])?
        .run()
        .await?;

    Ok(account)
}

/// Returns a unique username based on `preferred`, appending a random suffix if
/// the base username is already taken.
async fn unique_username(ctx: &RouteContext<()>, preferred: &str) -> Result<String> {
    let base = sanitize_username(preferred);
    if fetch_account_by_username(ctx, &base).await?.is_none() {
        return Ok(base);
    }

    let suffix: u32 = rand::random();
    let candidate = format!("{}_{suffix:08x}", &base[..base.len().min(24)]);
    if fetch_account_by_username(ctx, &candidate).await?.is_none() {
        return Ok(candidate);
    }

    let suffix: u32 = rand::random();
    Ok(format!("{}_{suffix:08x}", &base[..base.len().min(24)]))
}

/// Normalizes a raw provider login into a valid username.
fn sanitize_username(login: &str) -> String {
    let mut out = String::with_capacity(login.len().min(32));
    for ch in login.chars().take(32) {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.chars().all(|c| c == '_') {
        out.clear();
        out.push_str("user");
    }
    out
}

/// Returns a list of all accounts.
pub async fn list_accounts(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let result = db(&ctx)?.prepare(SELECT_ACCOUNT_COLUMNS).all().await?;
    let accounts: Vec<Account> = result.results::<Account>()?;
    Response::from_json(&accounts)
}

/// Responds with the account matching the `:id` path parameter, or 404 if not found.
pub async fn read_account(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    match fetch_account(&ctx, &id).await? {
        Some(account) => Response::from_json(&account),
        None => Response::error("Not found", 404),
    }
}

/// Responds with the OAuth providers linked to the currently authenticated
/// account as a JSON array of provider names.
pub async fn list_linked_providers(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let providers = fetch_linked_providers(&ctx, &account.id).await?;
    Response::from_json(&serde_json::json!({ "providers": providers }))
}

/// Unlinks an OAuth provider from the currently authenticated account.
pub async fn unlink_provider(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    let provider = ctx.param("provider").cloned().unwrap_or_default();
    crate::api::accounts::unlink_oauth_account(&ctx, &account.id, &provider).await?;
    Response::empty()
}

/// Creates a new account from the request body. Restricted to admins.
pub async fn create_account(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    if account.role != "admin" {
        return Response::error("Forbidden", 403);
    }

    let payload: AccountPayload = req.json().await?;
    db(&ctx)?
        .prepare(
            "INSERT INTO accounts (id, username, display_name, email, role, bio, avatar_url, created_at, verified) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        )
        .bind(&[
            payload.id.into(),
            payload.username.into(),
            payload.display_name.into(),
            payload.email.into(),
            payload.role.into(),
            payload.bio.into(),
            js_option(payload.avatar_url),
            payload.created_at.into(),
            payload.verified.into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Replaces an existing account, identified by the `:id` path parameter.
/// Restricted to admins.
pub async fn update_account(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    if account.role != "admin" {
        return Response::error("Forbidden", 403);
    }

    let id = ctx.param("id").cloned().unwrap_or_default();
    let payload: AccountPayload = req.json().await?;
    db(&ctx)?
        .prepare(
            "UPDATE accounts \
             SET username = ?1, display_name = ?2, email = ?3, role = ?4, bio = ?5, avatar_url = ?6, created_at = ?7, verified = ?8 \
             WHERE id = ?9",
        )
        .bind(&[
            payload.username.into(),
            payload.display_name.into(),
            payload.email.into(),
            payload.role.into(),
            payload.bio.into(),
            js_option(payload.avatar_url),
            payload.created_at.into(),
            payload.verified.into(),
            id.into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Deletes the account identified by the `:id` path parameter.
/// Restricted to admins.
pub async fn delete_account(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    if account.role != "admin" {
        return Response::error("Forbidden", 403);
    }

    let id = ctx.param("id").cloned().unwrap_or_default();
    db(&ctx)?
        .prepare("DELETE FROM accounts WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;
    Response::empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_username_keeps_allowed_characters() {
        assert_eq!(sanitize_username("hello-world_123"), "hello-world_123");
    }

    #[test]
    fn sanitize_username_replaces_invalid_characters() {
        assert_eq!(sanitize_username("hello world@foo"), "hello_world_foo");
    }

    #[test]
    fn sanitize_username_falls_back_for_empty() {
        assert_eq!(sanitize_username(""), "user");
        assert_eq!(sanitize_username("!!!"), "user");
    }
}
