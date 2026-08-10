/// Backend route paths.
pub(crate) mod routes {
    pub(crate) const AUTH_PREFIX: &str = "/auth/";
    pub(crate) const ACCOUNTS: &str = "/data/accounts";
    pub(crate) const ACCOUNT: &str = "/data/accounts/:id";
    pub(crate) const PROJECTS: &str = "/data/projects";
    pub(crate) const PROJECT: &str = "/data/projects/:id";
    pub(crate) const PROJECT_FAVORITES: &str = "/data/projects/:id/favorites";
    pub(crate) const PROJECT_DOWNLOADS: &str = "/data/projects/:id/downloads";
    pub(crate) const PROJECT_IFC: &str = "/data/projects/:id/ifc";
    pub(crate) const PROJECT_VERSIONS: &str = "/data/projects/:id/versions";
    pub(crate) const PROJECT_VERSION: &str = "/data/projects/:id/versions/:version_id";
    pub(crate) const PROJECT_GLBS: &str = "/data/projects/:id/glb";
    pub(crate) const PROJECT_GLBS_METADATA: &str = "/data/projects/:id/glb-metadata";
    pub(crate) const IFCS: &str = "/data/ifcs/:version_id/:filename";
    pub(crate) const LOGIN_GITHUB: &str = "/login/github";
    pub(crate) const AUTH_GITHUB_CALLBACK: &str = "/auth/github/callback";
    pub(crate) const LOGIN_GOOGLE: &str = "/login/google";
    pub(crate) const AUTH_GOOGLE_CALLBACK: &str = "/auth/google/callback";
    pub(crate) const AUTH_LINKED_PROVIDERS: &str = "/auth/linked-providers";
    pub(crate) const AUTH_LINKED_PROVIDER: &str = "/auth/linked-providers/:provider";
    pub(crate) const AUTH_ME: &str = "/auth/me";
    pub(crate) const AUTH_ME_VIEWER_PREFERENCES: &str = "/auth/me/viewer-preferences";
    pub(crate) const AUTH_LOGOUT: &str = "/auth/logout";
}

mod utils;

mod cors;

mod guards;

mod api {
    pub mod accounts;
    pub mod auth;
    pub mod projects;
    pub mod session;
    pub mod turnstile;
}

use worker::{Context, Env, Method, Request, Response, Result, Router, event};

use crate::cors::{add_cors_headers, allowed_origin, cors_preflight, error_response_with_cors};

/// Builds the request router with all API routes registered.
///
/// Extracted so route wiring can be exercised in tests and so the entry point
/// stays focused on request dispatch.
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
        .post_async(
            routes::PROJECT_DOWNLOADS,
            api::projects::increment_project_downloads,
        )
        .post_async(routes::PROJECT_IFC, api::projects::upload_project_ifc)
        .delete_async(routes::PROJECT_IFC, api::projects::delete_project_ifc)
        .get_async(
            routes::PROJECT_VERSIONS,
            api::projects::list_project_versions,
        )
        .patch_async(
            routes::PROJECT_VERSION,
            api::projects::update_project_version,
        )
        .delete_async(
            routes::PROJECT_VERSION,
            api::projects::delete_project_version,
        )
        .get_async(routes::PROJECT_GLBS, api::projects::serve_project_glb)
        .get_async(
            routes::PROJECT_GLBS_METADATA,
            api::projects::serve_project_glb_metadata,
        )
        .post_async(routes::PROJECT_GLBS, api::projects::convert_project_glb)
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
        .get_async(
            routes::AUTH_ME_VIEWER_PREFERENCES,
            api::session::me_viewer_preferences,
        )
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
        Err(err) => error_response_with_cors(&err, &origin),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use api::accounts::{Account, Provider, Role};
    use api::projects::Project;
    use serde::Deserialize;
    use shared::tags::Tag;

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
        viewer_preferences: String,
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
        downloads: u64,
        #[serde(default)]
        favorites: String,
        timestamp: String,
    }

    #[test]
    fn router_builds_without_conflicting_routes() {
        let _ = build_router();
    }

    #[test]
    fn account_json_matches_frontend_contract() -> Result<(), serde_json::Error> {
        let account = Account {
            id: "acc-1".to_string(),
            username: "creator".to_string(),
            display_name: "Creator".to_string(),
            email: "creator@example.com".to_string(),
            role: Role::Creator,
            bio: "Bio".to_string(),
            avatar_url: Some("https://example.com/avatar.png".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            verified: 1,
            viewer_preferences: "{}".to_string(),
            provider: Provider::GitHub,
            provider_id: String::new(),
        };

        let json = serde_json::to_string(&account)?;
        let parsed: FrontendAccountData = serde_json::from_str(&json)?;

        assert_eq!(parsed.id, account.id);
        assert_eq!(parsed.username, account.username);
        assert_eq!(parsed.display_name, account.display_name);
        assert_eq!(parsed.email, account.email);
        assert_eq!(parsed.role, account.role.to_string());
        assert_eq!(parsed.bio, account.bio);
        assert_eq!(parsed.avatar_url, account.avatar_url);
        assert!(parsed.project_ids.is_empty());
        assert_eq!(parsed.created_at, account.created_at);
        assert_eq!(parsed.verified, account.verified);
        assert_eq!(parsed.viewer_preferences, account.viewer_preferences);
        assert_eq!(parsed.provider, account.provider.to_string());
        assert_eq!(parsed.provider_id, account.provider_id);
        Ok(())
    }

    #[test]
    fn project_json_matches_frontend_contract() -> Result<(), serde_json::Error> {
        let project = Project {
            id: "proj-1".to_string(),
            title: "Mountain Bike".to_string(),
            author: "TrailBlazer".to_string(),
            author_id: "acc-1".to_string(),
            author_username: "trailblazer".to_string(),
            collaborator_ids: vec!["acc-2".to_string()],
            description: "Extended description.".to_string(),
            tags: vec![Tag::ThreeDModel, Tag::Vehicle],
            downloads: 1200,
            favorites: vec!["fav-1".to_string()],
            timestamp: "2026-07-07T14:30:00Z".to_string(),
        };

        let json = serde_json::to_string(&project)?;
        let parsed: FrontendProjectData = serde_json::from_str(&json)?;

        assert_eq!(parsed.id, project.id);
        assert_eq!(parsed.title, project.title);
        assert_eq!(parsed.author, project.author);
        assert_eq!(parsed.author_id, project.author_id);
        assert_eq!(parsed.author_username, project.author_username);
        assert_eq!(parsed.collaborator_ids, "[\"acc-2\"]");
        assert_eq!(parsed.description, project.description);
        assert_eq!(parsed.tags, "[\"3d_model\",\"vehicle\"]");
        assert_eq!(parsed.downloads, project.downloads);
        assert_eq!(parsed.favorites, "[\"fav-1\"]");
        assert_eq!(parsed.timestamp, project.timestamp);
        Ok(())
    }
}
