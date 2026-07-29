#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Name of the D1 binding configured in `wrangler.toml`.
pub(crate) const DB_BINDING: &str = "DB";
/// Name of the R2 binding configured in `wrangler.toml` for project assets
/// (icons, IFC models, and future project files).
pub(crate) const PROJECT_ASSETS_R2_BINDING: &str = "PROJECT_ASSETS";

/// Origins allowed to call the API from a browser.
///
/// `http://localhost:8080` is included so developers can hit the local backend
/// directly (not only through Trunk's proxy) during local development. It is
/// never sent by a production deployment.
const ALLOWED_ORIGINS: &[&str] = &[
    "https://cadiotheka.com",
    "https://www.cadiotheka.com",
    "http://localhost:8080",
];

/// Backend route paths.
pub(crate) mod routes {
    pub(crate) const AUTH_PREFIX: &str = "/auth/";
    pub(crate) const ACCOUNTS: &str = "/data/accounts";
    pub(crate) const ACCOUNT: &str = "/data/accounts/:id";
    pub(crate) const PROJECTS: &str = "/data/projects";
    pub(crate) const PROJECT: &str = "/data/projects/:id";
    pub(crate) const PROJECT_FAVORITES: &str = "/data/projects/:id/favorites";
    pub(crate) const PROJECT_IFC: &str = "/data/projects/:id/ifc";
    pub(crate) const PROJECT_GLBS: &str = "/data/projects/:id/glb";
    pub(crate) const IFCS: &str = "/data/ifcs/:project_id/:filename";
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
/// Returns the original response unchanged if its headers are immutable
/// (e.g. redirects created with `Response::redirect`). Propagates any other
/// header error so CORS misconfigurations are not silently ignored.
fn add_cors_headers(mut resp: Response, origin: &str) -> Result<Response> {
    let headers = resp.headers_mut();
    if let Err(err) = headers.set("Access-Control-Allow-Origin", origin) {
        if is_immutable_headers_error(&err) {
            return Ok(resp);
        }
        return Err(err);
    }
    headers.set(
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, PATCH, DELETE, OPTIONS",
    )?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;
    headers.set("Access-Control-Allow-Credentials", "true")?;
    Ok(resp)
}

/// Heuristic to detect the immutable-headers error returned by `web_sys` when
/// trying to mutate a response with a guard such as a redirect.
fn is_immutable_headers_error(err: &worker::Error) -> bool {
    let message = err.to_string().to_lowercase();
    message.contains("immutable")
        || message.contains("guard")
        || message.contains("headers are immutable")
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
    select_allowed_origin(req.headers().get("Origin").ok().flatten().as_deref())
}

/// Selects an allowed origin from an optional request origin header value.
///
/// If the origin is in [`ALLOWED_ORIGINS`] it is returned verbatim; otherwise
/// the first allowed origin is returned as a safe default.
fn select_allowed_origin(origin: Option<&str>) -> String {
    origin
        .and_then(|value| {
            ALLOWED_ORIGINS
                .iter()
                .find(|&&allowed| allowed == value)
                .map(|_| value)
        })
        .unwrap_or_else(|| ALLOWED_ORIGINS[0])
        .to_string()
}

/// Builds the request router with all API routes registered.
///
/// Extracted so route wiring can be exercised in tests and so the entry point
/// stays focused on CORS and environment handling.
pub fn build_router() -> Router<'static, ()> {
    Router::new()
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
        .post_async(routes::PROJECT_IFC, api::projects::upload_project_ifc)
        .delete_async(routes::PROJECT_IFC, api::projects::delete_project_ifc)
        .get_async(routes::PROJECT_GLBS, api::projects::serve_project_glb)
        .get_async(routes::IFCS, api::projects::serve_ifc)
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
}

#[event(fetch)]
async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let router = build_router();
    let origin = allowed_origin(&req);

    if req.method() == Method::Options {
        return cors_preflight(&origin);
    }

    let path = req.path();
    let is_data_route = path.starts_with("/data/");
    let is_login_route = path.starts_with("/login/");

    let result = router.run(req, env).await;

    match result {
        Ok(resp) => {
            let is_redirect = (300..400).contains(&resp.status_code());
            if is_data_route
                || is_login_route
                || (!is_redirect && path.starts_with(routes::AUTH_PREFIX))
            {
                Ok(add_cors_headers(resp, &origin)?)
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

#[cfg(test)]
mod tests {
    use super::*;
    use api::accounts::Account;
    use api::projects::Project;
    use serde::Deserialize;

    /// Frontend-compatible representation of an account response.
    ///
    /// The backend `Account` struct omits `project_ids`, `provider`, and
    /// `provider_id`, which the frontend supplies via `#[serde(default)]`. This
    /// struct mirrors the frontend contract so we can verify the JSON shape
    /// round-trips correctly.
    #[derive(Debug, Deserialize)]
    struct FrontendAccountData {
        id: String,
        username: String,
        display_name: String,
        email: String,
        role: String,
        bio: String,
        avatar_url: Option<String>,
        #[serde(default)]
        project_ids: Vec<String>,
        created_at: String,
        verified: i32,
        #[serde(default)]
        provider: String,
        #[serde(default)]
        provider_id: String,
    }

    /// Frontend-compatible representation of a project response.
    ///
    /// The backend serializes the JSON-string columns exactly as the frontend
    /// expects to deserialize them. This struct confirms the contract without
    /// importing the frontend crate into the backend tests.
    #[derive(Debug, Deserialize)]
    struct FrontendProjectData {
        id: String,
        title: String,
        author: String,
        author_id: String,
        author_username: String,
        #[serde(default)]
        collaborator_ids: String,
        description: String,
        tags: String,
        supported_platforms: String,
        downloads: u64,
        #[serde(default)]
        favorites: String,
        timestamp: String,
        #[serde(default)]
        icon_url: Option<String>,
        #[serde(default)]
        ifc_url: Option<String>,
    }

    #[test]
    fn router_builds_without_conflicting_routes() {
        let _ = build_router();
    }

    #[test]
    fn allowed_origin_selects_production_origin() {
        assert_eq!(
            select_allowed_origin(Some("https://cadiotheka.com")),
            "https://cadiotheka.com"
        );
        assert_eq!(
            select_allowed_origin(Some("https://www.cadiotheka.com")),
            "https://www.cadiotheka.com"
        );
    }

    #[test]
    fn allowed_origin_selects_localhost() {
        assert_eq!(
            select_allowed_origin(Some("http://localhost:8080")),
            "http://localhost:8080"
        );
    }

    #[test]
    fn allowed_origin_falls_back_for_missing_or_unknown_origin() {
        assert_eq!(select_allowed_origin(None), "https://cadiotheka.com");
        assert_eq!(
            select_allowed_origin(Some("https://evil.com")),
            "https://cadiotheka.com"
        );
    }

    #[test]
    fn account_json_matches_frontend_contract() {
        let account = Account {
            id: "acc-1".to_string(),
            username: "creator".to_string(),
            display_name: "Creator".to_string(),
            email: "creator@example.com".to_string(),
            role: "creator".to_string(),
            bio: "Bio".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            verified: 1,
        };

        let json = serde_json::to_string(&account).expect("account serializes");
        let parsed: FrontendAccountData =
            serde_json::from_str(&json).expect("account matches frontend contract");

        assert_eq!(parsed.id, account.id);
        assert_eq!(parsed.username, account.username);
        assert_eq!(parsed.display_name, account.display_name);
        assert_eq!(parsed.email, account.email);
        assert_eq!(parsed.role, account.role);
        assert_eq!(parsed.bio, account.bio);
        assert_eq!(parsed.avatar_url, account.avatar_url);
        assert!(parsed.project_ids.is_empty());
        assert_eq!(parsed.created_at, account.created_at);
        assert_eq!(parsed.verified, account.verified);
        assert!(parsed.provider.is_empty());
        assert!(parsed.provider_id.is_empty());
    }

    #[test]
    fn project_json_matches_frontend_contract() {
        let project = Project {
            id: "proj-1".to_string(),
            title: "Mountain Bike".to_string(),
            author: "TrailBlazer".to_string(),
            author_id: "acc-1".to_string(),
            author_username: "trailblazer".to_string(),
            collaborator_ids: vec!["acc-2".to_string()],
            description: "Extended description.".to_string(),
            tags: vec!["3d_model".to_string(), "vehicle".to_string()],
            supported_platforms: vec!["blender".to_string(), "freecad".to_string()],
            downloads: 1200,
            favorites: vec!["fav-1".to_string()],
            timestamp: "2026-07-07T14:30:00Z".to_string(),
            ifc_url: Some("ifcs/proj-1/model.ifc".to_string()),
        };

        let json = serde_json::to_string(&project).expect("project serializes");
        let parsed: FrontendProjectData =
            serde_json::from_str(&json).expect("project matches frontend contract");

        assert_eq!(parsed.id, project.id);
        assert_eq!(parsed.title, project.title);
        assert_eq!(parsed.author, project.author);
        assert_eq!(parsed.author_id, project.author_id);
        assert_eq!(parsed.author_username, project.author_username);
        assert_eq!(parsed.collaborator_ids, "[\"acc-2\"]");
        assert_eq!(parsed.description, project.description);
        assert_eq!(parsed.tags, "[\"3d_model\",\"vehicle\"]");
        assert_eq!(parsed.supported_platforms, "[\"blender\",\"freecad\"]");
        assert_eq!(parsed.downloads, project.downloads);
        assert_eq!(parsed.favorites, "[\"fav-1\"]");
        assert_eq!(parsed.timestamp, project.timestamp);
        assert_eq!(parsed.icon_url, None);
        assert_eq!(parsed.ifc_url, project.ifc_url);
    }
}
