//! Shared account types used by the frontend and backend.

use serde::{Deserialize, Serialize};

/// The set of roles an account can have.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Regular content creator.
    Creator,
    /// Platform administrator.
    Admin,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Creator => write!(f, "creator"),
            Self::Admin => write!(f, "admin"),
        }
    }
}

/// Normalizes a raw provider login into a valid username.
///
/// Keeps alphanumeric characters, hyphens, and underscores; replaces all
/// other characters with underscores. Falls back to `user` when the result
/// would be all underscores.
pub fn sanitize_username(login: &str) -> String {
    let mut out = String::with_capacity(login.len().min(32));
    for ch in login.chars().take(32) {
        if ch.is_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.chars().all(|c| c == '_') {
        out.clear();
        out.push_str("user");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_serializes_to_snake_case() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_string(&Role::Creator)?, "\"creator\"");
        assert_eq!(serde_json::to_string(&Role::Admin)?, "\"admin\"");
        Ok(())
    }

    #[test]
    fn role_displays_as_wire_id() {
        assert_eq!(Role::Creator.to_string(), "creator");
        assert_eq!(Role::Admin.to_string(), "admin");
    }

    #[test]
    fn sanitize_username_keeps_allowed_characters() {
        assert_eq!(sanitize_username("hello-world_123"), "hello-world_123");
    }

    #[test]
    fn sanitize_username_replaces_invalid_characters() {
        assert_eq!(sanitize_username("hello world@foo"), "hello_world_foo");
    }

    #[test]
    fn sanitize_username_falls_back_for_empty() {
        assert_eq!(sanitize_username(""), "user");
        assert_eq!(sanitize_username("!!!"), "user");
    }
}
