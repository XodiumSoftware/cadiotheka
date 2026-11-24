//! Browser storage keys shared across the app.
//!
//! Values are stored via `gloo-storage` (`LocalStorage` / `SessionStorage`),
//! which JSON-serializes them. A prefix is applied to all keys to avoid
//! collisions with other apps on the same origin.

/// Prefix applied to all stored keys to avoid collisions with other apps on
/// the same origin.
const PREFIX: &str = "cadiotheka.";

/// Builds the fully qualified storage key for the given logical name.
pub fn key(name: &str) -> String {
    format!("{PREFIX}{name}")
}

/// Storage key recording which sign-in method the user last completed
/// successfully. One of `"github"`, `"google"`, or `"passkey"`.
pub const LAST_LOGIN_METHOD_KEY: &str = "last_login_method";

/// Storage key used as a one-shot marker set just before navigating away to an
/// OAuth provider. When the user lands back on the app with a valid session,
/// the current-user context promotes it to [`LAST_LOGIN_METHOD_KEY`] and clears
/// this marker.
pub const PENDING_LOGIN_METHOD_KEY: &str = "pending_login_method";

/// Persists a value to browser storage, logging instead of failing.
///
/// Preferences are conveniences, so a failed write (Safari private mode, a
/// full quota, or a serialization error) must not break the interaction that
/// triggered it. The failure is surfaced in the console rather than discarded
/// so a silently non-persisting preference stays diagnosable.
pub fn save<S>(key: &str, value: &impl serde::Serialize)
where
    S: gloo_storage::Storage,
{
    if let Err(err) = S::set(key, value) {
        leptos::logging::error!("failed to persist {key}: {err}");
    }
}
