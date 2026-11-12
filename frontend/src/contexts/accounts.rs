use crate::data::error::RequestError;
use crate::data::{AccountData, fetch_accounts};
use leptos::prelude::*;

/// Provides the list of accounts fetched from the backend.
#[derive(Clone, Copy)]
pub struct AccountsContext {
    /// The accounts currently loaded; empty until the first fetch completes.
    pub accounts: Signal<Vec<AccountData>>,
    /// Setter used to replace the loaded account list, e.g. after a retry.
    pub set_accounts: WriteSignal<Vec<AccountData>>,
    /// Whether a fetch of `/data/accounts` is currently in flight.
    pub is_loading: Signal<bool>,
    /// Setter that toggles the network activity indication.
    pub set_is_loading: WriteSignal<bool>,
    /// The last fetch failure, or `None` when the list loaded successfully.
    pub error: Signal<Option<RequestError>>,
    set_error: WriteSignal<Option<RequestError>>,
}

impl AccountsContext {
    /// Spawns a fetch of `/data/accounts` that updates the list, loading flag,
    /// and error state on completion.
    fn spawn_fetch(
        set_accounts: WriteSignal<Vec<AccountData>>,
        set_is_loading: WriteSignal<bool>,
        set_error: WriteSignal<Option<RequestError>>,
    ) {
        leptos::task::spawn_local(async move {
            set_is_loading.set(true);
            set_error.set(None);
            match fetch_accounts().await {
                Ok(fetched) => set_accounts.set(fetched),
                Err(err) => {
                    web_sys::console::error_1(
                        &format!("Failed to load accounts: {}", err.message()).into(),
                    );
                    set_error.set(Some(err));
                }
            }
            set_is_loading.set(false);
        });
    }

    /// Provide an empty account list and kick off a fetch from `/data/accounts`.
    pub fn provide() {
        let (accounts, set_accounts) = signal(Vec::new());
        let (is_loading, set_is_loading) = signal(true);
        let (error, set_error) = signal(None);
        provide_context(Self {
            accounts: accounts.into(),
            set_accounts,
            is_loading: is_loading.into(),
            set_is_loading,
            error: error.into(),
            set_error,
        });

        Self::spawn_fetch(set_accounts, set_is_loading, set_error);
    }

    /// Re-fetches the account list after a failure.
    pub fn retry(&self) {
        Self::spawn_fetch(self.set_accounts, self.set_is_loading, self.set_error);
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        expect_context::<Self>()
    }
}
