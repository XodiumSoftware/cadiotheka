use crate::contexts::AccountsContext;
use crate::data::AccountData;
use leptos::prelude::*;

/// Provides and reads the currently logged-in user.
///
/// For development this defaults to the first account returned by the backend.
/// Until the account list has loaded, the context holds a placeholder account.
#[derive(Clone, Copy)]
pub struct CurrentUserContext {
    pub account: Signal<AccountData>,
    pub set_account: WriteSignal<AccountData>,
}

impl CurrentUserContext {
    /// Provide a current-user context that starts as a placeholder and updates
    /// to the first fetched account once `/data/accounts` returns.
    pub fn provide_with_default() {
        let (account, set_account) = signal(AccountData::placeholder());
        provide_context(Self {
            account: account.into(),
            set_account,
        });

        let accounts = AccountsContext::use_context().accounts;
        leptos::task::spawn_local(async move {
            // Wait until accounts have been fetched, then pick the first one.
            // A real login system would replace this with the logged-in user.
            loop {
                let accounts = accounts.get_untracked();
                if !accounts.is_empty() {
                    set_account.set(accounts[0].clone());
                    break;
                }
                gloo_timers::future::sleep(std::time::Duration::from_millis(50)).await;
            }
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
