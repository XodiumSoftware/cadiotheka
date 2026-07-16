use serde::{Deserialize, Serialize};
use worker::*;

use crate::DB_BINDING;
use crate::api::session::require_account;

const SELECT_ACCOUNT_COLUMNS: &str = "SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified, provider, provider_id FROM accounts";

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
    pub provider: String,
    pub provider_id: String,
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
    pub provider: String,
    pub provider_id: String,
}

/// Returns the D1 database binding configured for this worker.
fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1(DB_BINDING)
}

/// Fetches a single account by id, returning `None` when no row matches.
async fn fetch_account(ctx: &RouteContext<()>, id: &str) -> Result<Option<Account>> {
    let result = db(ctx)?
        .prepare(format!("{SELECT_ACCOUNT_COLUMNS} WHERE id = ?1"))
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
            "{SELECT_ACCOUNT_COLUMNS} WHERE provider = ?1 AND provider_id = ?2"
        ))
        .bind(&[provider.into(), provider_id.into()])?
        .all()
        .await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    Ok(accounts.pop())
}

/// Fetches a single account by username, returning `None` when no row matches.
async fn fetch_account_by_username(
    ctx: &RouteContext<()>,
    username: &str,
) -> Result<Option<Account>> {
    let result = db(ctx)?
        .prepare(format!("{SELECT_ACCOUNT_COLUMNS} WHERE username = ?1"))
        .bind(&[username.into()])?
        .all()
        .await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    Ok(accounts.pop())
}

/// Inserts a new account from an OAuth login.
///
/// The username is made unique by appending a short random suffix if the
/// provider's preferred login is already taken.
pub async fn create_oauth_account(
    ctx: &RouteContext<()>,
    provider: &str,
    provider_id: &str,
    preferred_username: &str,
    display_name: &str,
    email: &str,
    avatar_url: Option<String>,
) -> Result<Account> {
    let id = uuid::Uuid::new_v4().to_string();
    let created_at = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|e| worker::Error::RustError(format!("failed to format timestamp: {e}")))?;
    let username = unique_username(ctx, preferred_username).await?;

    let account = Account {
        id: id.clone(),
        username: username.clone(),
        display_name: display_name.to_string(),
        email: email.to_string(),
        role: "creator".to_string(),
        bio: String::new(),
        avatar_url,
        created_at,
        verified: 1,
        provider: provider.to_string(),
        provider_id: provider_id.to_string(),
    };

    db(ctx)?
        .prepare(
            "INSERT INTO accounts (id, username, display_name, email, role, bio, avatar_url, created_at, verified, provider, provider_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .bind(&[
            id.into(),
            username.into(),
            account.display_name.clone().into(),
            account.email.clone().into(),
            account.role.clone().into(),
            account.bio.clone().into(),
            account.avatar_url.clone().into(),
            account.created_at.clone().into(),
            account.verified.into(),
            account.provider.clone().into(),
            account.provider_id.clone().into(),
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

    // Extremely unlikely collision: try once more with a different suffix.
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

/// Responds with a JSON array of all accounts.
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

/// Creates a new account from the request body. Restricted to admins.
pub async fn create_account(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let account = require_account(&req, &ctx).await?;
    if account.role != "admin" {
        return Response::error("Forbidden", 403);
    }

    let payload: AccountPayload = req.json().await?;
    db(&ctx)?
        .prepare(
            "INSERT INTO accounts (id, username, display_name, email, role, bio, avatar_url, created_at, verified, provider, provider_id) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        )
        .bind(&[
            payload.id.into(),
            payload.username.into(),
            payload.display_name.into(),
            payload.email.into(),
            payload.role.into(),
            payload.bio.into(),
            payload.avatar_url.into(),
            payload.created_at.into(),
            payload.verified.into(),
            payload.provider.into(),
            payload.provider_id.into(),
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
             SET username = ?1, display_name = ?2, email = ?3, role = ?4, bio = ?5, avatar_url = ?6, created_at = ?7, verified = ?8, provider = ?9, provider_id = ?10 \
             WHERE id = ?11",
        )
        .bind(&[
            payload.username.into(),
            payload.display_name.into(),
            payload.email.into(),
            payload.role.into(),
            payload.bio.into(),
            payload.avatar_url.into(),
            payload.created_at.into(),
            payload.verified.into(),
            payload.provider.into(),
            payload.provider_id.into(),
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
