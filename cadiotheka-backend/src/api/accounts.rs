use serde::{Deserialize, Serialize};
use worker::*;

use crate::DB_BINDING;

const SELECT_ACCOUNT_COLUMNS: &str = "SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified FROM accounts";

/// A Cadiotheka account stored in D1.
#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: String,
    pub bio: String,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub verified: bool,
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
    pub verified: bool,
}

/// Returns the D1 database binding configured for this worker.
fn db(ctx: &RouteContext<()>) -> Result<D1Database> {
    ctx.env.d1(DB_BINDING)
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

/// Creates a new account from the request body.
pub async fn create_account(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
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
            payload.avatar_url.into(),
            payload.created_at.into(),
            (payload.verified as i32).into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Replaces an existing account, identified by the `:id` path parameter.
pub async fn update_account(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
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
            payload.avatar_url.into(),
            payload.created_at.into(),
            (payload.verified as i32).into(),
            id.into(),
        ])?
        .run()
        .await?;
    Response::empty()
}

/// Deletes the account identified by the `:id` path parameter.
pub async fn delete_account(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    db(&ctx)?
        .prepare("DELETE FROM accounts WHERE id = ?1")
        .bind(&[id.into()])?
        .run()
        .await?;
    Response::empty()
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
