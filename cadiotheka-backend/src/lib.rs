/// Name of the D1 binding configured in `wrangler.toml`.
pub(crate) const DB_BINDING: &str = "DB";

mod api {
    pub mod accounts;
    pub mod projects;
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
        .get_async("/api/projects", api::projects::list_projects)
        .post_async("/api/projects", api::projects::create_project)
        .get_async("/api/projects/:id", api::projects::read_project)
        .put_async("/api/projects/:id", api::projects::update_project)
        .delete_async("/api/projects/:id", api::projects::delete_project)
        .run(req, env)
        .await
}
