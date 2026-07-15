use crate::data::{AccountData, load_accounts};
use leptos::prelude::*;

/// Provides and reads the currently logged-in user.
///
/// For development this defaults to the first account in the fixture.
#[derive(Clone, Copy)]
pub struct CurrentUserContext {
    pub account: Signal<AccountData>,
    pub set_account: WriteSignal<AccountData>,
}

impl CurrentUserContext {
    /// Provide a default current-user context.
    ///
    /// Panics if the accounts fixture is empty.
    pub fn provide_with_default() {
        let accounts = load_accounts();
        let initial = accounts
            .into_iter()
            .next()
            .expect("accounts fixture should contain at least one account");
        let (account, set_account) = signal(initial);
        provide_context(Self {
            account: account.into(),
            set_account,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
