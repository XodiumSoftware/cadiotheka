use crate::data::{AccountData, fetch_accounts};
use leptos::prelude::*;

/// Provides the list of accounts fetched from the backend.
#[derive(Clone, Copy)]
pub struct AccountsContext {
    pub accounts: Signal<Vec<AccountData>>,
    pub set_accounts: WriteSignal<Vec<AccountData>>,
    pub is_loading: Signal<bool>,
    pub set_is_loading: WriteSignal<bool>,
}

impl AccountsContext {
    /// Provide an empty account list and kick off a fetch from `/data/accounts`.
    pub fn provide() {
        let (accounts, set_accounts) = signal(Vec::new());
        let (is_loading, set_is_loading) = signal(true);
        provide_context(Self {
            accounts: accounts.into(),
            set_accounts,
            is_loading: is_loading.into(),
            set_is_loading,
        });

        leptos::task::spawn_local(async move {
            let fetched = fetch_accounts().await;
            set_accounts.set(fetched);
            set_is_loading.set(false);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
