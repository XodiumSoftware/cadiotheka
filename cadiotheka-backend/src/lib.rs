mod api {
    pub mod accounts;
}

use worker::*;

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();

    router
        .get_async("/api/accounts", api::accounts::list_accounts)
        .get_async("/api/accounts/:id", api::accounts::get_account)
        .run(req, env)
        .await
}
