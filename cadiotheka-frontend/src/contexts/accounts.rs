use crate::data::{AccountData, fetch_accounts};
use leptos::prelude::*;

/// Provides the list of accounts fetched from the backend.
#[derive(Clone, Copy)]
pub struct AccountsContext {
    pub accounts: Signal<Vec<AccountData>>,
    pub set_accounts: WriteSignal<Vec<AccountData>>,
}

impl AccountsContext {
    /// Provide an empty account list and kick off a fetch from `/api/accounts`.
    pub fn provide() {
        let (accounts, set_accounts) = signal(Vec::new());
        provide_context(Self {
            accounts: accounts.into(),
            set_accounts,
        });

        leptos::task::spawn_local(async move {
            let fetched = fetch_accounts().await;
            set_accounts.set(fetched);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
