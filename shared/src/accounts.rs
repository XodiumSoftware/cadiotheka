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

/// Controls which parts of a project an account may modify.
///
/// The backend enforces this; the frontend uses it to decide which editing
/// affordances to render, so the two cannot disagree about who is allowed to
/// do what.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectPermission {
    /// Nobody is granted the requested change.
    None,
    /// The account may edit the project's content fields.
    Content,
    /// The account may edit content *and* the collaborator list.
    Ownership,
}

impl ProjectPermission {
    /// Resolves the permission an account has on a project.
    ///
    /// Collaborators can edit a project's content, but only the author (or an
    /// admin) may change who the collaborators are. Granting that to
    /// collaborators would let any of them add accomplices or remove the
    /// others, so the collaborator list stays behind an ownership check.
    #[must_use]
    pub fn project(
        role: Role,
        account_id: &str,
        author_id: &str,
        collaborators: &[String],
    ) -> Self {
        if role == Role::Admin || account_id == author_id {
            return Self::Ownership;
        }
        if collaborators.iter().any(|id| id == account_id) {
            return Self::Content;
        }
        Self::None
    }

    /// Returns `true` when content fields may be edited.
    #[must_use]
    pub const fn can_edit_content(self) -> bool {
        matches!(self, Self::Content | Self::Ownership)
    }

    /// Returns `true` when the collaborator list may be edited.
    #[must_use]
    pub const fn can_edit_collaborators(self) -> bool {
        matches!(self, Self::Ownership)
    }
}

/// Normalizes a raw provider login into a valid username.
///
/// Keeps alphanumeric characters, hyphens, and underscores; replaces all
/// other characters with underscores. Falls back to `user` when the result
/// would be all underscores.
#[must_use]
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

    #[test]
    fn author_and_admin_may_edit_collaborators() {
        let collaborators = vec!["acc-2".to_owned()];
        assert_eq!(
            ProjectPermission::project(Role::Creator, "acc-1", "acc-1", &collaborators),
            ProjectPermission::Ownership
        );
        assert_eq!(
            ProjectPermission::project(Role::Admin, "acc-9", "acc-1", &collaborators),
            ProjectPermission::Ownership
        );
    }

    #[test]
    fn collaborators_may_edit_content_but_not_collaborators() {
        let collaborators = vec!["acc-2".to_owned()];
        let permission =
            ProjectPermission::project(Role::Creator, "acc-2", "acc-1", &collaborators);
        assert!(permission.can_edit_content());
        assert!(!permission.can_edit_collaborators());
    }

    #[test]
    fn unrelated_accounts_are_denied() {
        let collaborators = vec!["acc-2".to_owned()];
        let permission =
            ProjectPermission::project(Role::Creator, "acc-3", "acc-1", &collaborators);
        assert_eq!(permission, ProjectPermission::None);
        assert!(!permission.can_edit_content());
        assert!(!permission.can_edit_collaborators());
    }
}
