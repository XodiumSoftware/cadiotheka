use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize, Deserialize, Debug)]
struct Account {
    id: String,
    username: String,
    display_name: String,
    email: String,
    role: String,
    bio: String,
    avatar_url: Option<String>,
    created_at: String,
    verified: bool,
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/api/accounts", |_, ctx| async move {
            let db = ctx.env.d1("DB")?;
            let statement = db.prepare("SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified FROM accounts");
            let result = statement.all().await?;
            let accounts: Vec<Account> = result.results::<Account>()?;
            Response::from_json(&accounts)
        })
        .get_async("/api/accounts/:id", |_, ctx| async move {
            let id = ctx.param("id").cloned().unwrap_or_default();
            let db = ctx.env.d1("DB")?;
            let statement = db.prepare("SELECT id, username, display_name, email, role, bio, avatar_url, created_at, verified FROM accounts WHERE id = ?1");
            let result = statement.bind(&[id.into()])?.all().await?;
            let mut accounts: Vec<Account> = result.results::<Account>()?;
            match accounts.pop() {
                Some(account) => Response::from_json(&account),
                None => Response::error("Not found", 404),
            }
        })
        .run(req, env)
        .await
}
