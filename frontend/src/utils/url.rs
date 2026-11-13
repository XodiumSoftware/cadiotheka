use shared::routes::{
    ACCOUNTS, AUTH_LINKED_PROVIDER, AUTH_LINKED_PROVIDERS, AUTH_LOGOUT, AUTH_ME,
    AUTH_ME_VIEWER_PREFERENCES, AUTH_PASSKEY, AUTH_PASSKEY_LOGIN_BEGIN, AUTH_PASSKEY_LOGIN_FINISH,
    AUTH_PASSKEY_REGISTER_BEGIN, AUTH_PASSKEY_REGISTER_FINISH, AUTH_PASSKEYS, IFCS, LOGIN_GITHUB,
    LOGIN_GOOGLE, PROJECT, PROJECT_DOWNLOADS, PROJECT_FAVORITES, PROJECT_GLBS,
    PROJECT_GLBS_METADATA, PROJECT_IFC, PROJECT_VERSION, PROJECT_VERSIONS, PROJECTS,
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
    let base = backend_origin();
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

/// Returns the full URL for a GitHub login endpoint in the root browsing
/// context.
///
/// OAuth cannot complete inside a frame: the provider sign-in page and the
/// callback's `window.location.replace` both refuse to be framed, and the
/// server-built `redirect_to` would point back into the embedding page's URL.
/// The frontend navigates the top-level window here.
pub fn github_login_url() -> String {
    root_context_url(&backend_url(LOGIN_GITHUB))
}

/// Returns the full URL for a Google login endpoint in the root browsing
/// context. See [`github_login_url`] for why frames are excluded.
pub fn google_login_url() -> String {
    root_context_url(&backend_url(LOGIN_GOOGLE))
}

/// Returns this site's absolute origin when the app was loaded inside a frame.
///
/// The embedded page reads `window.location` to build its own URLs, which inside
/// a frame is the embedder's URL rather than this site's. An absolute origin is
/// the only way for those URLs to point at the top-level browsing context.
fn root_context() -> Option<String> {
    let window = web_sys::window()?;
    let framed = window.top().ok().flatten().is_some_and(|top| {
        let same_window: &web_sys::Window = top.as_ref();
        !std::ptr::eq(same_window, &raw const window)
    });
    if !framed {
        return None;
    }
    window.location().origin().ok()
}

/// Prefixes a same-origin path with this site's origin when framed.
///
/// Absolute URLs are returned unchanged: prefixing `https://api.example/x`
/// with this origin would produce an unparseable concatenation that the
/// browser resolves as a relative path, so the request silently targets the
/// wrong host.
fn root_context_url(path: &str) -> String {
    match root_context() {
        Some(origin) => join_root_context(path, Some(&origin)),
        None => join_root_context(path, None),
    }
}

/// Joins a path or absolute URL against an optional root origin.
fn join_root_context(path: &str, origin: Option<&str>) -> String {
    if path.contains("://") {
        return path.to_string();
    }
    match origin {
        Some(origin) => format!("{origin}{path}"),
        None => path.to_string(),
    }
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
///
/// Like the login endpoints, this is resolved against the root context: the
/// backend redirects after clearing the session, and inside a frame that would
/// land the embedder's page in the embedder's own URL.
pub fn logout_url() -> String {
    root_context_url(&backend_url(AUTH_LOGOUT))
}

/// Returns the full URL for a single linked OAuth provider endpoint.
pub fn linked_provider_url(provider: &str) -> String {
    backend_url(&AUTH_LINKED_PROVIDER.replace(":provider", provider))
}

/// Returns the full URL for the passkey registration begin endpoint.
pub fn passkey_register_begin_url() -> String {
    backend_url(AUTH_PASSKEY_REGISTER_BEGIN)
}

/// Returns the full URL for the passkey registration finish endpoint.
pub fn passkey_register_finish_url() -> String {
    backend_url(AUTH_PASSKEY_REGISTER_FINISH)
}

/// Returns the full URL for the passkey login begin endpoint.
pub fn passkey_login_begin_url() -> String {
    backend_url(AUTH_PASSKEY_LOGIN_BEGIN)
}

/// Returns the full URL for the passkey login finish endpoint.
pub fn passkey_login_finish_url() -> String {
    backend_url(AUTH_PASSKEY_LOGIN_FINISH)
}

/// Returns the full URL for the passkey collection endpoint.
pub fn passkeys_url() -> String {
    backend_url(AUTH_PASSKEYS)
}

/// Returns the full URL for a single passkey endpoint.
pub fn passkey_url(id: &str) -> String {
    backend_url(&AUTH_PASSKEY.replace(":id", id))
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

/// Appends a safe `redirect_to` query parameter to a URL, using a safe return
/// target.
///
/// Relative paths are used on top-level views so the backend can send the
/// browser back to the current route. The absolute origin of this site is used
/// inside a frame, where the embedder's URL must never be the return target,
/// and during local development so the backend can send the browser back to the
/// Trunk dev server.
///
/// The separator is `?` or `&` depending on whether `base` already carries a
/// query string; appending a second `?` would corrupt the URL.
pub fn encode_redirect_url(base: &str) -> String {
    let redirect_to = root_context()
        .map(|origin| format!("{origin}/"))
        .or_else(|| web_sys::window().and_then(|w| w.location().href().ok()))
        .unwrap_or_else(|| "/".to_string());

    append_redirect_param(base, &redirect_to)
}

/// Appends an encoded `redirect_to` parameter to `base`, choosing `?` or `&`
/// depending on whether `base` already carries a query string. Appending a
/// second `?` corrupts the URL, which the browser then resolves as a relative
/// path and fails with a CORS error and no status code.
fn append_redirect_param(base: &str, redirect_to: &str) -> String {
    let encoded = urlencoding::encode(redirect_to);
    let separator = if base.contains('?') { '&' } else { '?' };
    format!("{base}{separator}redirect_to={encoded}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_root_context_leaves_absolute_urls_untouched() {
        assert_eq!(
            join_root_context(
                "https://api.cadiotheka.com/login/github",
                Some("https://cadiotheka.com")
            ),
            "https://api.cadiotheka.com/login/github"
        );
    }

    #[test]
    fn join_root_context_prefixes_relative_paths() {
        assert_eq!(
            join_root_context("/login/github", Some("https://cadiotheka.com")),
            "https://cadiotheka.com/login/github"
        );
    }

    #[test]
    fn append_redirect_param_uses_question_mark_without_query() {
        assert_eq!(
            append_redirect_param("https://api.cadiotheka.com/login/github", "/"),
            "https://api.cadiotheka.com/login/github?redirect_to=%2F"
        );
    }

    #[test]
    fn append_redirect_param_uses_ampersand_when_query_present() {
        assert_eq!(
            append_redirect_param(
                "https://api.cadiotheka.com/login/github?scope=read",
                "https://cadiotheka.com/"
            ),
            "https://api.cadiotheka.com/login/github?scope=read&redirect_to=https%3A%2F%2Fcadiotheka.com%2F"
        );
    }

    #[test]
    fn append_redirect_param_never_produces_a_second_question_mark() {
        let url = append_redirect_param("https://api.cadiotheka.com/login/github?x=1", "/");
        assert_eq!(url.matches('?').count(), 1);
    }
}
