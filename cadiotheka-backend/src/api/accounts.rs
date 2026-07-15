use serde::{Deserialize, Serialize};
use worker::*;

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

const SELECT_ACCOUNT_COLUMNS: &str = "SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified FROM accounts";

/// Responds with a JSON array of all accounts.
pub async fn list_accounts(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db = ctx.env.d1("DB")?;
    let statement = db.prepare(SELECT_ACCOUNT_COLUMNS);
    let result = statement.all().await?;
    let accounts: Vec<Account> = result.results::<Account>()?;
    Response::from_json(&accounts)
}

/// Responds with the account matching the `:id` path parameter, or 404 if not found.
pub async fn get_account(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let id = ctx.param("id").cloned().unwrap_or_default();
    let db = ctx.env.d1("DB")?;
    let statement = db.prepare(format!("{SELECT_ACCOUNT_COLUMNS} WHERE id = ?1"));
    let result = statement.bind(&[id.into()])?.all().await?;
    let mut accounts: Vec<Account> = result.results::<Account>()?;
    match accounts.pop() {
        Some(account) => Response::from_json(&account),
        None => Response::error("Not found", 404),
    }
}
