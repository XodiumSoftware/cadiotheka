//! Shared API route paths used by the frontend and backend.
//!
//! Keeping route paths in one place avoids mismatches between the backend
//! router and the frontend fetch helpers.

/// Base prefix for public data endpoints.
pub const DATA_PREFIX: &str = "/data";
/// Prefix for auth endpoints.
pub const AUTH_PREFIX: &str = "/auth/";

/// Account collection endpoint.
pub const ACCOUNTS: &str = "/data/accounts";
/// Single account endpoint, with `:id` placeholder.
pub const ACCOUNT: &str = "/data/accounts/:id";

/// Project collection endpoint.
pub const PROJECTS: &str = "/data/projects";
/// Single project endpoint, with `:id` placeholder.
pub const PROJECT: &str = "/data/projects/:id";
/// Project favorites toggle endpoint.
pub const PROJECT_FAVORITES: &str = "/data/projects/:id/favorites";
/// Project downloads counter endpoint.
pub const PROJECT_DOWNLOADS: &str = "/data/projects/:id/downloads";
/// Project IFC upload/delete endpoint.
pub const PROJECT_IFC: &str = "/data/projects/:id/ifc";
/// Project version collection endpoint.
pub const PROJECT_VERSIONS: &str = "/data/projects/:id/versions";
/// Single project version endpoint, with `:version_id` placeholder.
pub const PROJECT_VERSION: &str = "/data/projects/:id/versions/:version_id";
/// Converted GLB endpoint.
pub const PROJECT_GLBS: &str = "/data/projects/:id/glb";
/// Per-primitive GLB metadata endpoint.
pub const PROJECT_GLBS_METADATA: &str = "/data/projects/:id/glb-metadata";
/// IFC file download endpoint, with `:version_id` and `:filename` placeholders.
pub const IFCS: &str = "/data/ifcs/:version_id/:filename";

/// GitHub login initiation endpoint.
pub const LOGIN_GITHUB: &str = "/login/github";
/// GitHub OAuth callback endpoint.
pub const AUTH_GITHUB_CALLBACK: &str = "/auth/github/callback";
/// Google login initiation endpoint.
pub const LOGIN_GOOGLE: &str = "/login/google";
/// Google OAuth callback endpoint.
pub const AUTH_GOOGLE_CALLBACK: &str = "/auth/google/callback";
/// Linked OAuth providers collection endpoint.
pub const AUTH_LINKED_PROVIDERS: &str = "/auth/linked-providers";
/// Single linked OAuth provider endpoint, with `:provider` placeholder.
pub const AUTH_LINKED_PROVIDER: &str = "/auth/linked-providers/:provider";
/// Current authenticated account endpoint.
pub const AUTH_ME: &str = "/auth/me";
/// Viewer preferences endpoint.
pub const AUTH_ME_VIEWER_PREFERENCES: &str = "/auth/me/viewer-preferences";
/// Logout endpoint.
pub const AUTH_LOGOUT: &str = "/auth/logout";
