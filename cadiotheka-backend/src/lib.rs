/// Name of the D1 binding configured in `wrangler.toml`.
pub(crate) const DB_BINDING: &str = "DB";

/// Origins allowed to call the API from a browser.
const ALLOWED_ORIGINS: &[&str] = &["https://cadiotheka.com", "https://www.cadiotheka.com"];

mod api {
    pub mod accounts;
    pub mod projects;
}

use worker::*;

/// Adds CORS headers to a response so the frontend (served from a different
/// origin) can read the JSON body.
fn add_cors_headers(mut resp: Response, origin: &str) -> Result<Response> {
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    Ok(resp)
}

/// Responds to CORS preflight requests.
fn cors_preflight(origin: &str) -> Result<Response> {
    let mut resp = Response::empty()?;
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    Ok(resp)
}

/// Returns the request origin if it is in the allowed list, otherwise the
/// first allowed origin as a safe fallback.
fn allowed_origin(req: &Request) -> String {
    req.headers()
        .get("Origin")
        .ok()
        .flatten()
        .and_then(|origin| {
            ALLOWED_ORIGINS
                .iter()
                .find(|&&allowed| allowed == origin)
                .map(|_| origin)
        })
        .unwrap_or_else(|| ALLOWED_ORIGINS[0].to_string())
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = Router::new();
    let origin = allowed_origin(&req);

    if req.method() == Method::Options {
        return cors_preflight(&origin);
    }

    let result = router
        .get_async("/data/accounts", api::accounts::list_accounts)
        .post_async("/data/accounts", api::accounts::create_account)
        .get_async("/data/accounts/:id", api::accounts::read_account)
        .put_async("/data/accounts/:id", api::accounts::update_account)
        .delete_async("/data/accounts/:id", api::accounts::delete_account)
        .get_async("/data/projects", api::projects::list_projects)
        .post_async("/data/projects", api::projects::create_project)
        .get_async("/data/projects/:id", api::projects::read_project)
        .put_async("/data/projects/:id", api::projects::update_project)
        .delete_async("/data/projects/:id", api::projects::delete_project)
        .run(req, env)
        .await;

    match result {
        Ok(resp) => add_cors_headers(resp, &origin),
        Err(err) => {
            let mut resp = Response::error(err.to_string(), 500)?;
            let headers = resp.headers_mut();
            headers.set("Access-Control-Allow-Origin", &origin)?;
            Ok(resp)
        }
    }
}
