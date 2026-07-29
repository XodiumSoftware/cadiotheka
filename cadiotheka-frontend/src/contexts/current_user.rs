#![allow(clippy::missing_errors_doc)]

use crate::data::AccountData;
use crate::data::error::RequestError;
use crate::utils::auth_url;
use gloo_net::http::Request;
use leptos::prelude::*;
use web_sys::RequestCredentials;

/// Provides and reads the currently logged-in user.
///
/// On startup this context fetches `/auth/me`. If the user is not authenticated
/// the signal holds `None`.
#[derive(Clone, Copy)]
pub struct CurrentUserContext {
    pub account: Signal<Option<AccountData>>,
    pub set_account: WriteSignal<Option<AccountData>>,
    pub is_loading: Signal<bool>,
    pub set_is_loading: WriteSignal<bool>,
}

impl CurrentUserContext {
    /// Provide a current-user context and fetch the authenticated user.
    pub fn provide() {
        let (account, set_account) = signal::<Option<AccountData>>(None);
        let (is_loading, set_is_loading) = signal(true);
        provide_context(Self {
            account: account.into(),
            set_account,
            is_loading: is_loading.into(),
            set_is_loading,
        });

        leptos::task::spawn_local(async move {
            let fetched = fetch_current_user().await.unwrap_or(None);
            set_account.set(fetched);
            set_is_loading.set(false);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}

/// Fetch the currently authenticated account from the backend.
///
/// Returns `Ok(None)` when the user is not logged in. Any other failure is
/// returned as a [`RequestError`] so the caller can decide how to report it.
pub async fn fetch_current_user() -> Result<Option<AccountData>, RequestError> {
    let url = auth_url("/me");
    match Request::get(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            let parsed = serde_json::from_str::<MeResponse>(&body).map_err(|err| {
                RequestError::Parse(format!(
                    "Failed to parse /auth/me response from {url}: {err} (status={status}, body={body:?})"
                ))
            })?;
            Ok(Some(parsed.account))
        }
        Ok(response) if response.status() == 401 => Ok(None),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch current user from {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch current user from {url}: {err}"
        ))),
    }
}

#[derive(serde::Deserialize, Debug)]
struct MeResponse {
    account: AccountData,
}

/// Fetch the OAuth provider names linked to the currently authenticated
/// account.
///
/// Returns a list of provider names on success or a [`RequestError`] on
/// failure.
pub async fn fetch_linked_providers() -> Result<Vec<String>, RequestError> {
    let url = auth_url("/linked-providers");
    match Request::get(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let body = response.text().await.unwrap_or_default();
            serde_json::from_str::<LinkedProvidersResponse>(&body)
                .map(|parsed| parsed.providers)
                .map_err(|err| {
                    RequestError::Parse(format!(
                        "Failed to parse linked providers response from {url}: {err} (body={body:?})"
                    ))
                })
        }
        Ok(response) if response.status() == 401 => Ok(Vec::new()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch linked providers from {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch linked providers from {url}: {err}"
        ))),
    }
}

/// Unlinks an OAuth provider from the currently authenticated account.
///
/// Returns `Ok(())` if the provider was successfully unlinked.
pub async fn unlink_provider(provider: &str) -> Result<(), RequestError> {
    let url = auth_url(&format!("/linked-providers/{provider}"));
    match Request::delete(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to unlink provider at {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to unlink provider at {url}: {err}"
        ))),
    }
}

#[derive(serde::Deserialize, Debug)]
struct LinkedProvidersResponse {
    providers: Vec<String>,
}

/// Maximum length for a user-written bio, matching GitHub's profile bio limit.
const MAX_BIO_LENGTH: usize = 160;

/// Updates the current user's bio on the backend.
///
/// Returns the new bio on success or a [`RequestError`] describing what went
/// wrong.
pub async fn update_bio(new_bio: String) -> Result<String, RequestError> {
    if new_bio.len() > MAX_BIO_LENGTH {
        return Err(RequestError::Serialize(format!(
            "Bio must be {MAX_BIO_LENGTH} characters or fewer"
        )));
    }

    let url = auth_url("/me");
    let request = Request::put(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::json!({ "bio": new_bio }).to_string())
        .map_err(|err| {
            RequestError::BuildRequest(format!("Failed to build bio update request: {err}"))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(new_bio),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to update bio at {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to update bio at {url}: {err}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_starts_unauthenticated() {
        let placeholder = AccountData::placeholder();
        assert!(placeholder.id.is_empty());
    }
}
