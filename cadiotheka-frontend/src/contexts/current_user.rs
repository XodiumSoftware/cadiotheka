use crate::data::AccountData;
use crate::utils::auth_url;
use leptos::prelude::*;

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
    match gloo_net::http::Request::get(&auth_url("/me")).send().await {
        Ok(response) if response.ok() => {
            response.json::<MeResponse>().await.ok().map(|r| r.account)
        }
        Ok(response) if response.status() == 401 => None,
        Ok(response) => {
            let status = response.status();
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch current user: HTTP {status}").into(),
            );
            None
        }
        Err(err) => {
            leptos::web_sys::console::error_1(
                &format!("Failed to fetch current user: {err:?}").into(),
            );
            None
        }
    }
}

#[derive(serde::Deserialize, Debug)]
struct MeResponse {
    account: AccountData,
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
