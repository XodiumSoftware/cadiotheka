/// Name of the D1 binding configured in `wrangler.toml`.
pub(crate) const DB_BINDING: &str = "DB";

mod api {
    pub mod accounts;
}

use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/api/accounts", api::accounts::list_accounts)
        .post_async("/api/accounts", api::accounts::create_account)
        .get_async("/api/accounts/:id", api::accounts::read_account)
        .put_async("/api/accounts/:id", api::accounts::update_account)
        .delete_async("/api/accounts/:id", api::accounts::delete_account)
        .run(req, env)
        .await
}
