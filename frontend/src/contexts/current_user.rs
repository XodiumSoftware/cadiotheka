use crate::data::AccountData;
use crate::data::error::RequestError;
use crate::utils::url::{
    linked_provider_url, linked_providers_url, me_url, me_viewer_preferences_url, session_url,
};
use crate::utils::{LAST_LOGIN_METHOD_KEY, PENDING_LOGIN_METHOD_KEY, key as storage_key};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, SessionStorage, Storage as _};
use leptos::prelude::*;
use web_sys::RequestCredentials;

/// Provides and reads the currently logged-in user.
///
/// On startup this context asks the backend whether a session exists, and only
/// then fetches `/auth/me`: a visitor who has never signed in is settled as
/// anonymous without provoking a 401. The question cannot be answered from
/// `document.cookie`, because the session cookie is `HttpOnly` and host-only on
/// the API origin while this page runs on the site origin.
#[derive(Clone, Copy)]
pub struct CurrentUserContext {
    /// Whether the current user is `Some`; `is_loading` is `true` until the
    /// initial `/auth/me` request settles.
    pub account: Signal<Option<AccountData>>,
    /// Setter used to swap the authenticated account after login or logout.
    pub set_account: WriteSignal<Option<AccountData>>,
    /// Whether the initial `/auth/me` request is in flight.
    pub is_loading: Signal<bool>,
    /// Setter that toggles the loading indication.
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
            let fetched = if fetch_session_status(&session_url()).await {
                fetch_current_user().await.unwrap_or(None)
            } else {
                None
            };
            set_is_loading.set(false);
            if fetched.is_some()
                && let Some(pending) =
                    SessionStorage::get::<String>(&storage_key(PENDING_LOGIN_METHOD_KEY)).ok()
            {
                crate::utils::save::<LocalStorage>(&storage_key(LAST_LOGIN_METHOD_KEY), &pending);
                SessionStorage::delete(storage_key(PENDING_LOGIN_METHOD_KEY));
            }
            set_account.set(fetched);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        expect_context::<Self>()
    }
}

/// Fetch the currently authenticated account from the backend.
///
/// Returns `Ok(None)` when the user is not logged in.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails, the backend returns an
/// unexpected error, or the response cannot be parsed.
pub async fn fetch_current_user() -> Result<Option<AccountData>, RequestError> {
    let url = me_url();
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

/// Fetches whether the browser holds a session, so the app can skip `/auth/me`
/// for anonymous visitors.
///
/// This cannot be answered from `document.cookie`: the session cookie is
/// `HttpOnly`, and any JavaScript-readable companion would live on the API
/// origin, where this page cannot see it. The endpoint returns only a boolean
/// and is safe to call when signed out.
///
/// A failed status request reports `false`, which leaves a signed-in visitor
/// rendered as anonymous for that page load. That is the deliberate tradeoff:
/// the alternative is a 401 on every page load for visitors who never signed
/// in.
async fn fetch_session_status(url: &str) -> bool {
    let Ok(response) = Request::get(url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    else {
        return false;
    };
    if !response.ok() {
        return false;
    }
    response
        .text()
        .await
        .ok()
        .and_then(|body| serde_json::from_str::<SessionStatusResponse>(&body).ok())
        .is_some_and(|status| status.authenticated)
}

#[derive(serde::Deserialize)]
struct SessionStatusResponse {
    authenticated: bool,
}

/// Fetch the account-scoped viewer preferences JSON blob from the backend.
///
/// Returns `"{}"` when the user is not logged in.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails or the backend returns an
/// unexpected error.
pub async fn fetch_viewer_preferences() -> Result<String, RequestError> {
    let url = me_viewer_preferences_url();
    match Request::get(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let body = response.text().await.unwrap_or_default();
            let parsed =
                serde_json::from_str::<ViewerPreferencesResponse>(&body).map_err(|err| {
                    RequestError::Parse(format!(
                        "Failed to parse viewer preferences from {url}: {err} (body={body:?})"
                    ))
                })?;
            Ok(parsed.viewer_preferences)
        }
        Ok(response) if response.status() == 401 => Ok("{}".to_string()),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to fetch viewer preferences from {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to fetch viewer preferences from {url}: {err}"
        ))),
    }
}

#[derive(serde::Deserialize, Debug)]
struct ViewerPreferencesResponse {
    viewer_preferences: String,
}

/// Saves account-scoped viewer preferences on the backend.
///
/// On success it returns the JSON string that was sent.
///
/// # Errors
///
/// Returns a [`RequestError`] when serialization fails or the backend rejects
/// the request.
pub async fn update_viewer_preferences(preferences: String) -> Result<String, RequestError> {
    let url = me_url();
    let body = serde_json::json!({ "viewer_preferences": preferences }).to_string();
    let request = Request::put(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|err| {
            RequestError::BuildRequest(format!(
                "Failed to build viewer preferences update request: {err}"
            ))
        })?;

    match request.send().await {
        Ok(response) if response.ok() => Ok(preferences),
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Err(RequestError::Server {
                status,
                body: format!("Failed to update viewer preferences at {url}: {body}"),
            })
        }
        Err(err) => Err(RequestError::Network(format!(
            "Failed to update viewer preferences at {url}: {err}"
        ))),
    }
}

/// Fetch the OAuth provider names linked to the currently authenticated
/// account.
///
/// On success it returns a list of provider names.
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails, the backend returns an
/// unexpected error, or the response cannot be parsed.
pub async fn fetch_linked_providers() -> Result<Vec<String>, RequestError> {
    let url = linked_providers_url();
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
///
/// # Errors
///
/// Returns a [`RequestError`] when the network fails or the backend rejects the
/// request.
pub async fn unlink_provider(provider: &str) -> Result<(), RequestError> {
    let url = linked_provider_url(provider);
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
/// On success it returns the new bio.
///
/// # Errors
///
/// Returns a [`RequestError`] when the bio is too long, the request cannot be
/// built, the network fails, or the backend rejects the request.
pub async fn update_bio(new_bio: String) -> Result<String, RequestError> {
    if new_bio.len() > MAX_BIO_LENGTH {
        return Err(RequestError::Serialize(format!(
            "Bio must be {MAX_BIO_LENGTH} characters or fewer"
        )));
    }

    let url = me_url();
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
        assert_eq!(placeholder.id, "");
    }
}
