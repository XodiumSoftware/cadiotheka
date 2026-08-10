use shared::routes::{
    ACCOUNTS, AUTH_LINKED_PROVIDER, AUTH_LINKED_PROVIDERS, AUTH_LOGOUT, AUTH_ME,
    AUTH_ME_VIEWER_PREFERENCES, IFCS, LOGIN_GITHUB, LOGIN_GOOGLE, PROJECT, PROJECT_DOWNLOADS,
    PROJECT_FAVORITES, PROJECT_GLBS, PROJECT_GLBS_METADATA, PROJECT_IFC, PROJECT_VERSION,
    PROJECT_VERSIONS, PROJECTS,
};

/// Backend API origin.
///
/// In release builds the frontend is served from `cadiotheka.com` and talks
/// directly to `api.cadiotheka.com`. In debug builds Trunk proxies requests to
/// the local backend, so an empty origin is used to keep URLs relative.
const fn backend_origin() -> &'static str {
    if cfg!(debug_assertions) {
        ""
    } else {
        "https://api.cadiotheka.com"
    }
}

/// Builds a full backend URL from a route path.
fn backend_url(path: &str) -> String {
    let base = format!("{}{}", backend_origin(), shared::routes::DATA_PREFIX);
    if path.starts_with('/') {
        format!("{base}{path}")
    } else {
        format!("{base}/{path}")
    }
}

/// Returns the full URL for the accounts endpoint.
pub fn accounts_url() -> String {
    backend_url(ACCOUNTS)
}

/// Returns the full URL for the projects endpoint.
pub fn projects_url() -> String {
    backend_url(PROJECTS)
}

/// Returns the full URL for a single project endpoint.
pub fn project_url(id: &str) -> String {
    backend_url(&PROJECT.replace(":id", id))
}

/// Returns the full URL for a project's favorites endpoint.
pub fn project_favorites_url(id: &str) -> String {
    backend_url(&PROJECT_FAVORITES.replace(":id", id))
}

/// Returns the full URL for a project's downloads endpoint.
pub fn project_downloads_url(id: &str) -> String {
    backend_url(&PROJECT_DOWNLOADS.replace(":id", id))
}

/// Returns the full URL for a project's IFC upload/delete endpoint.
pub fn project_ifc_url(id: &str) -> String {
    backend_url(&PROJECT_IFC.replace(":id", id))
}

/// Returns the full URL for a project's version collection endpoint.
pub fn project_versions_url(id: &str) -> String {
    backend_url(&PROJECT_VERSIONS.replace(":id", id))
}

/// Returns the full URL for a single project version endpoint.
pub fn project_version_url(project_id: &str, version_id: &str) -> String {
    backend_url(
        &PROJECT_VERSION
            .replace(":id", project_id)
            .replace(":version_id", version_id),
    )
}

/// Returns the full URL for a project's converted GLB endpoint.
pub fn project_glb_url(id: &str) -> String {
    backend_url(&PROJECT_GLBS.replace(":id", id))
}

/// Returns the full URL for a project's GLB metadata endpoint.
pub fn project_glb_metadata_url(id: &str) -> String {
    backend_url(&PROJECT_GLBS_METADATA.replace(":id", id))
}

/// Returns the full URL for an IFC file download endpoint.
pub fn ifc_url(version_id: &str, filename: &str) -> String {
    backend_url(
        &IFCS
            .replace(":version_id", version_id)
            .replace(":filename", filename),
    )
}

/// Returns the full URL for a GitHub login endpoint.
pub fn github_login_url() -> String {
    backend_url(LOGIN_GITHUB)
}

/// Returns the full URL for a Google login endpoint.
pub fn google_login_url() -> String {
    backend_url(LOGIN_GOOGLE)
}

/// Returns the full URL for the linked OAuth providers endpoint.
pub fn linked_providers_url() -> String {
    backend_url(AUTH_LINKED_PROVIDERS)
}

/// Returns the full URL for the current authenticated account endpoint.
pub fn me_url() -> String {
    backend_url(AUTH_ME)
}

/// Returns the full URL for the viewer preferences endpoint.
pub fn me_viewer_preferences_url() -> String {
    backend_url(AUTH_ME_VIEWER_PREFERENCES)
}

/// Returns the full URL for the logout endpoint.
pub fn logout_url() -> String {
    backend_url(AUTH_LOGOUT)
}

/// Returns the full URL for a single linked OAuth provider endpoint.
pub fn linked_provider_url(provider: &str) -> String {
    backend_url(&AUTH_LINKED_PROVIDER.replace(":provider", provider))
}

/// Returns the full URL for a backend API path (`/data/...`).
#[deprecated(note = "prefer the typed route helpers in this module")]
pub fn api_url(path: &str) -> String {
    backend_url(path)
}

/// Returns the full URL for an auth endpoint (`/auth/...`).
#[deprecated(note = "prefer the typed route helpers in this module")]
pub fn auth_url(path: &str) -> String {
    backend_url(path)
}

/// Returns the full URL for an OAuth login provider endpoint (`/login/...`).
#[deprecated(note = "prefer github_login_url or google_login_url")]
pub fn login_url(path: &str) -> String {
    backend_url(&format!("/login/{path}"))
}

/// Appends a safe `redirect_to` query parameter to a URL, using the current
/// browser location as the return target. Relative paths are used in release
/// builds; the full URL is used during local development so the backend can
/// send the browser back to the Trunk dev server.
pub fn encode_redirect_url(base: &str) -> String {
    let redirect_to = leptos::web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_else(|| "/".to_string());

    let encoded = urlencoding::encode(&redirect_to);
    format!("{base}?redirect_to={encoded}")
}
