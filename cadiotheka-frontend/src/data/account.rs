use crate::utils::api_url;
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
        }
    }
}

/// Fetch accounts from the backend API.
///
/// On failure it logs to the browser console and returns an empty vector so
/// the UI can keep running with a graceful fallback.
pub async fn fetch_accounts() -> Vec<AccountData> {
    let url = api_url("/accounts");
    match gloo_net::http::Request::get(&url).send().await {
        Ok(response) if response.ok() => {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<Vec<AccountData>>(&text) {
                Ok(data) => data,
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse accounts JSON from {url}: {err:?} (status={status}, body={text:?})"
                        )
                        .into(),
                    );
                    Vec::new()
                }
            }
        }
        Ok(response) => {
            let status = response.status();
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch accounts from {url}: HTTP {status}").into(),
            );
            Vec::new()
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch accounts from {url}: {err:?}").into(),
            );
            Vec::new()
        }
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
        }
    }

    #[test]
    fn account_serializes_and_deserializes() {
        let account = sample_account();
        let json = serde_json::to_string(&account).expect("account serializes");
        let decoded: AccountData = serde_json::from_str(&json).expect("account deserializes");
        assert_eq!(decoded, account);
    }

    #[test]
    fn account_role_serializes_to_snake_case() {
        assert_eq!(
            serde_json::to_string(&AccountRole::Creator).unwrap(),
            "\"creator\""
        );
        assert_eq!(
            serde_json::to_string(&AccountRole::Admin).unwrap(),
            "\"admin\""
        );
    }

    #[test]
    fn account_deserializes_missing_optional_fields_with_defaults() {
        let json = r#"{"id":"acc-1","username":"user","display_name":"User","email":"u@example.com","role":"creator","created_at":"2025-01-01T00:00:00Z"}"#;
        let account: AccountData = serde_json::from_str(json).unwrap();
        assert_eq!(account.id, "acc-1");
        assert!(account.bio.is_empty());
        assert!(account.project_ids.is_empty());
        assert_eq!(account.verified, 0);
        assert!(account.provider.is_empty());
        assert!(account.provider_id.is_empty());
        assert!(account.avatar_url.is_none());
    }

    #[test]
    fn account_verified_integer_roundtrips() {
        let account = sample_account();
        let json = serde_json::to_string(&account).unwrap();
        assert!(json.contains("\"verified\":1"));
        let decoded: AccountData = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.verified, 1);
    }

    #[test]
    fn placeholder_account_is_empty() {
        let account = AccountData::placeholder();
        assert!(account.id.is_empty());
        assert!(account.username.is_empty());
    }
}
