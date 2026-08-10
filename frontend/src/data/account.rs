use crate::data::error::RequestError;
use crate::utils::accounts_url;
use serde::{Deserialize, Serialize};

/// Account role for a registered user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, derive_more::Display)]
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
    ///
    /// The backend stores this as a `SQLite` integer, so it is deserialized as
    /// an `i32` (0 = false, 1 = true).
    #[serde(default)]
    pub verified: i32,
    /// OAuth provider used to create this account (e.g. "github", "google").
    #[serde(default)]
    pub provider: String,
    /// Provider-scoped unique identifier for this account.
    #[serde(default)]
    pub provider_id: String,
    /// JSON blob of account-scoped viewer preferences.
    #[serde(default = "default_viewer_preferences")]
    pub viewer_preferences: String,
}

fn default_viewer_preferences() -> String {
    "{}".to_string()
}

impl AccountData {
    /// Returns a placeholder account used while the real data is still loading.
    pub fn placeholder() -> Self {
        Self {
            id: String::new(),
            username: String::new(),
            display_name: String::new(),
            email: String::new(),
            role: AccountRole::Creator,
            bio: String::new(),
            avatar_url: None,
            project_ids: Vec::new(),
            created_at: time::OffsetDateTime::UNIX_EPOCH,
            verified: 0,
            provider: String::new(),
            provider_id: String::new(),
            viewer_preferences: "{}".to_string(),
        }
    }
}

/// Fetch accounts from the backend API.
///
/// On success it returns a list of accounts.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails, the backend rejects the
/// request, or the response cannot be parsed.
pub async fn fetch_accounts() -> Result<Vec<AccountData>, RequestError> {
    let url = accounts_url();
    match gloo_net::http::Request::get(&url).send().await {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Vec<AccountData>>(&text)
                .map_err(|err| {
                    RequestError::Parse(format!(
                        "Failed to parse accounts JSON from {url}: {err:?} (status={status}, body={text:?})"
                    ))
                })
        }
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch accounts from {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch accounts from {url}: {err:?}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    fn sample_account() -> AccountData {
        AccountData {
            id: "8af81bd9-b70a-4d64-89e9-83bbc4e0297d".to_owned(),
            username: "TrailBlazer".to_owned(),
            display_name: "Trail Blazer".to_owned(),
            email: "trail@example.com".to_owned(),
            role: AccountRole::Creator,
            bio: "Outdoor gear and mechanical models.".to_owned(),
            avatar_url: None,
            project_ids: vec!["71e3dcb4-f52a-4ebc-bd1e-7052a8d5e5d2".to_owned()],
            created_at: datetime!(2025-03-10 12:00:00 UTC),
            verified: 1,
            provider: "seed".to_owned(),
            provider_id: "seed_8af81bd9-b70a-4d64-89e9-83bbc4e0297d".to_owned(),
            viewer_preferences: "{}".to_owned(),
        }
    }

    #[test]
    fn account_serializes_and_deserializes() -> Result<(), serde_json::Error> {
        let account = sample_account();
        let json = serde_json::to_string(&account)?;
        let decoded: AccountData = serde_json::from_str(&json)?;
        assert_eq!(decoded, account);
        Ok(())
    }

    #[test]
    fn account_role_serializes_to_snake_case() -> Result<(), serde_json::Error> {
        assert_eq!(serde_json::to_string(&AccountRole::Creator)?, "\"creator\"");
        assert_eq!(serde_json::to_string(&AccountRole::Admin)?, "\"admin\"");
        Ok(())
    }

    #[test]
    fn account_role_displays_as_human_label() {
        assert_eq!(AccountRole::Creator.to_string(), "Creator");
        assert_eq!(AccountRole::Admin.to_string(), "Admin");
    }

    #[test]
    fn account_deserializes_missing_optional_fields_with_defaults() -> Result<(), serde_json::Error>
    {
        let json = r#"{"id":"acc-1","username":"user","display_name":"User","email":"u@example.com","role":"creator","created_at":"2025-01-01T00:00:00Z"}"#;
        let account: AccountData = serde_json::from_str(json)?;
        assert_eq!(account.id, "acc-1");
        assert!(account.bio.is_empty());
        assert!(account.project_ids.is_empty());
        assert_eq!(account.verified, 0);
        assert!(account.provider.is_empty());
        assert!(account.provider_id.is_empty());
        assert!(account.avatar_url.is_none());
        assert_eq!(account.viewer_preferences, "{}");
        Ok(())
    }

    #[test]
    fn account_viewer_preferences_defaults_to_empty_object() -> Result<(), serde_json::Error> {
        let json = r#"{"id":"acc-1","username":"user","display_name":"User","email":"u@example.com","role":"creator","created_at":"2025-01-01T00:00:00Z"}"#;
        let account: AccountData = serde_json::from_str(json)?;
        assert_eq!(account.viewer_preferences, "{}");
        Ok(())
    }

    #[test]
    fn account_verified_integer_roundtrips() -> Result<(), serde_json::Error> {
        let account = sample_account();
        let json = serde_json::to_string(&account)?;
        assert!(json.contains("\"verified\":1"));
        let decoded: AccountData = serde_json::from_str(&json)?;
        assert_eq!(decoded.verified, 1);
        Ok(())
    }

    #[test]
    fn placeholder_account_is_empty() {
        let account = AccountData::placeholder();
        assert!(account.id.is_empty());
        assert!(account.username.is_empty());
    }
}
