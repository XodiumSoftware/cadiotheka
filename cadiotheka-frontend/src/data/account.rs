use serde::{Deserialize, Serialize};

/// Account role for a registered user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountRole {
    /// Regular content creator.
    Creator,
    /// Platform administrator.
    Admin,
}

/// A registered user account.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct AccountData {
    /// Unique account identifier.
    pub id: String,
    /// Public username (used in URLs and card attribution).
    pub username: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Contact email address.
    pub email: String,
    /// Account role.
    pub role: AccountRole,
    /// Short public bio.
    #[serde(default)]
    pub bio: String,
    /// Optional avatar URL.
    pub avatar_url: Option<String>,
    /// IDs of projects owned by this account.
    #[serde(default)]
    pub project_ids: Vec<String>,
    /// Timestamp when the account was created.
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: time::OffsetDateTime,
    /// Whether the account has been verified.
    #[serde(default)]
    pub verified: bool,
}

/// Load the embedded accounts fixture.
pub fn load_accounts() -> Vec<AccountData> {
    let fixture: super::AccountsFixture =
        serde_json::from_str(include_str!("../../test_data/accounts.json"))
            .expect("accounts fixture is valid JSON");
    fixture.accounts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_accounts_returns_entries() {
        let accounts = load_accounts();
        assert!(
            !accounts.is_empty(),
            "fixture should contain at least one account"
        );
    }
}
