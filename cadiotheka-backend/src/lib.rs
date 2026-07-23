#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Name of the D1 binding configured in `wrangler.toml`.
pub(crate) const DB_BINDING: &str = "DB";
/// Name of the R2 binding configured in `wrangler.toml` for project icons.
pub(crate) const ICONS_R2_BINDING: &str = "PI";

/// Origins allowed to call the API from a browser.
const ALLOWED_ORIGINS: &[&str] = &["https://cadiotheka.com", "https://www.cadiotheka.com"];

/// Backend route paths.
pub(crate) mod routes {
    pub(crate) const AUTH_PREFIX: &str = "/auth/";
    pub(crate) const ACCOUNTS: &str = "/data/accounts";
    pub(crate) const ACCOUNT: &str = "/data/accounts/:id";
    pub(crate) const PROJECTS: &str = "/data/projects";
    pub(crate) const PROJECT: &str = "/data/projects/:id";
    pub(crate) const PROJECT_FAVORITES: &str = "/data/projects/:id/favorites";
    pub(crate) const PROJECT_ICON: &str = "/data/projects/:id/icon";
    pub(crate) const ICONS: &str = "/data/icons/:project_id/:icon_id";
    pub(crate) const LOGIN_GITHUB: &str = "/login/github";
    pub(crate) const AUTH_GITHUB_CALLBACK: &str = "/auth/github/callback";
    pub(crate) const LOGIN_GOOGLE: &str = "/login/google";
    pub(crate) const AUTH_GOOGLE_CALLBACK: &str = "/auth/google/callback";
    pub(crate) const AUTH_LINKED_PROVIDERS: &str = "/auth/linked-providers";
    pub(crate) const AUTH_LINKED_PROVIDER: &str = "/auth/linked-providers/:provider";
    pub(crate) const AUTH_ME: &str = "/auth/me";
    pub(crate) const AUTH_LOGOUT: &str = "/auth/logout";
}

mod utils;

mod api {
    pub mod accounts;
    pub mod auth;
    pub mod projects;
    pub mod session;
}

use worker::{
    Context, Env, Headers, Method, Request, Response, ResponseBody, ResponseBuilder, Result,
    Router, event,
};

/// Adds CORS headers to a response so the frontend (served from a different
/// origin) can read the JSON body.
///
/// Some response types (notably redirects created with `Response::redirect`)
/// have immutable headers. In that case the original response is returned
/// unchanged.
fn add_cors_headers(mut resp: Response, origin: &str) -> Response {
    {
        let headers = resp.headers_mut();
        let _ = headers.set("Access-Control-Allow-Origin", origin);
        let _ = headers.set(
            "Access-Control-Allow-Methods",
            "GET, POST, PUT, PATCH, DELETE, OPTIONS",
        );
        let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
        let _ = headers.set("Access-Control-Allow-Credentials", "true");
    }
    resp
}

/// Responds to CORS preflight requests.
fn cors_preflight(origin: &str) -> Result<Response> {
    let mut resp = Response::empty()?;
    let headers = resp.headers_mut();
    headers.set("Access-Control-Allow-Origin", origin)?;
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    headers.set("Access-Control-Allow-Credentials", "true")?;
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
    let is_login_route = path.starts_with("/login/");

    let result = router
        .get_async(routes::ACCOUNTS, api::accounts::list_accounts)
        .post_async(routes::ACCOUNTS, api::accounts::create_account)
        .get_async(routes::ACCOUNT, api::accounts::read_account)
        .put_async(routes::ACCOUNT, api::accounts::update_account)
        .delete_async(routes::ACCOUNT, api::accounts::delete_account)
        .get_async(routes::PROJECTS, api::projects::list_projects)
        .post_async(routes::PROJECTS, api::projects::create_project)
        .get_async(routes::PROJECT, api::projects::read_project)
        .post_async(
            routes::PROJECT_FAVORITES,
            api::projects::toggle_project_favorite,
        )
        .post_async(routes::PROJECT_ICON, api::projects::upload_project_icon)
        .get_async(routes::ICONS, api::projects::serve_icon)
        .patch_async(routes::PROJECT, api::projects::patch_project)
        .put_async(routes::PROJECT, api::projects::update_project)
        .delete_async(routes::PROJECT, api::projects::delete_project)
        .get_async(routes::LOGIN_GITHUB, api::auth::github_login)
        .get_async(routes::AUTH_GITHUB_CALLBACK, api::auth::github_callback)
        .get_async(routes::LOGIN_GOOGLE, api::auth::google_login)
        .get_async(routes::AUTH_GOOGLE_CALLBACK, api::auth::google_callback)
        .get_async(
            routes::AUTH_LINKED_PROVIDERS,
            api::accounts::list_linked_providers,
        )
        .delete_async(routes::AUTH_LINKED_PROVIDER, api::accounts::unlink_provider)
        .get_async(routes::AUTH_ME, api::session::me)
        .put_async(routes::AUTH_ME, api::session::update_me)
        .get_async(routes::AUTH_LOGOUT, api::session::logout)
        .run(req, env)
        .await;

    match result {
        Ok(resp) => {
            let is_redirect = (300..400).contains(&resp.status_code());
            if is_data_route
                || is_login_route
                || (!is_redirect && path.starts_with(routes::AUTH_PREFIX))
            {
                Ok(add_cors_headers(resp, &origin))
            } else {
                Ok(resp)
            }
        }
        Err(err) => {
            let headers = Headers::new();
            headers.set("Content-Type", "text/plain")?;
            let _ = headers.set("Access-Control-Allow-Origin", &origin);
            let _ = headers.set("Access-Control-Allow-Credentials", "true");
            let _ = headers.set(
                "Access-Control-Allow-Methods",
                "GET, POST, PUT, PATCH, DELETE, OPTIONS",
            );
            let _ = headers.set("Access-Control-Allow-Headers", "Content-Type");
            Ok(ResponseBuilder::new()
                .with_status(500)
                .with_headers(headers)
                .body(ResponseBody::Body(err.to_string().into())))
        }
    }
}
