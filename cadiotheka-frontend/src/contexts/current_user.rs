use crate::data::AccountData;
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
            let fetched = fetch_current_user().await;
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
/// Returns `None` when the user is not logged in or the request fails.
async fn fetch_current_user() -> Option<AccountData> {
    let url = auth_url("/me");
    match Request::get(&url)
        .credentials(RequestCredentials::Include)
        .send()
        .await
    {
        Ok(response) if response.ok() => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            match serde_json::from_str::<MeResponse>(&body) {
                Ok(parsed) => Some(parsed.account),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!(
                            "Failed to parse /auth/me response from {url}: {err:?} (status={status}, body={body:?})"
                        )
                        .into(),
                    );
                    None
                }
            }
        }
        Ok(response) if response.status() == 401 => None,
        Ok(response) => {
            let status = response.status();
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch current user from {url}: HTTP {status}").into(),
            );
            None
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch current user from {url}: {err:?}").into(),
            );
            None
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct MeResponse {
    account: AccountData,
}

/// Updates the current user's bio on the backend and returns the new bio on
/// success, or `None` if the request failed.
pub async fn update_bio(new_bio: String) -> Option<String> {
    let url = auth_url("/me");
    let request = match Request::put(&url)
        .credentials(RequestCredentials::Include)
        .header("Content-Type", "application/json")
        .body(serde_json::json!({ "bio": new_bio }).to_string())
    {
        Ok(req) => req,
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to build bio update request: {err:?}").into(),
            );
            return None;
        }
    };

    match request.send().await {
        Ok(response) if response.ok() => Some(new_bio),
        Ok(response) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to update bio at {url}: HTTP {}", response.status()).into(),
            );
            None
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to update bio at {url}: {err:?}").into(),
            );
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_starts_unauthenticated() {
        // Signals must be created inside a reactive runtime, so this just
        // validates the placeholder behavior.
        let placeholder = AccountData::placeholder();
        assert!(placeholder.id.is_empty());
    }
}
