//! Shared origin constants used when validating post-auth redirects.
//!
//! Both the backend redirect validator and any frontend redirect logic should
//! agree on which origins are safe.

/// Production origins allowed for post-auth browser redirects.
pub const ALLOWED_REDIRECT_ORIGINS: &[&str] =
    &["https://cadiotheka.com", "https://www.cadiotheka.com"];

/// Localhost origins allowed for non-HTTPS development requests.
pub const ALLOWED_LOCALHOST_ORIGINS: &[&str] = &["http://localhost:8080", "http://localhost:8787"];
