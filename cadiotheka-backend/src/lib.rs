/// Name of the D1 binding configured in `wrangler.toml`.
pub(crate) const DB_BINDING: &str = "DB";

/// Origins allowed to call the API from a browser.
const ALLOWED_ORIGINS: &[&str] = &["https://cadiotheka.com", "https://www.cadiotheka.com"];

mod utils;

mod api {
    pub mod accounts;
    pub mod auth;
    pub mod projects;
    pub mod session;
}

use worker::*;

/// Adds CORS headers to a response so the frontend (served from a different
/// origin) can read the JSON body.
///
/// Some response types (notably redirects created with `Response::redirect`)
/// have immutable headers. In that case the original response is returned
/// unchanged.
fn add_cors_headers(mut resp: Response, origin: &str) -> Result<Response> {
    {
        let headers = resp.headers_mut();
        if headers.set("Access-Control-Allow-Origin", origin).is_err() {
            return Ok(resp);
        }
        if headers
            .set(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, DELETE, OPTIONS",
            )
            .is_err()
        {
            return Ok(resp);
        }
        if headers
            .set("Access-Control-Allow-Headers", "Content-Type")
            .is_err()
        {
            return Ok(resp);
        }
    }
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

    let path = req.path();
    let is_data_route = path.starts_with("/data/");

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
        .get_async("/auth/github", api::auth::github_login)
        .get_async("/auth/github/callback", api::auth::github_callback)
        .get_async("/auth/google", api::auth::google_login)
        .get_async("/auth/google/callback", api::auth::google_callback)
        .get_async("/auth/me", api::session::me)
        .get_async("/auth/logout", api::session::logout)
        .run(req, env)
        .await;

    match result {
        Ok(resp) => {
            // Auth routes are same-origin and use redirects with immutable
            // headers, so only attach CORS headers to data API responses.
            if is_data_route {
                add_cors_headers(resp, &origin)
            } else {
                Ok(resp)
            }
        }
        Err(err) => {
            let mut resp = Response::error(err.to_string(), 500)?;
            let headers = resp.headers_mut();
            let _ = headers.set("Access-Control-Allow-Origin", &origin);
            Ok(resp)
        }
    }
}
