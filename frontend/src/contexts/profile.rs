use crate::data::AccountData;
use leptos::prelude::*;

/// Provides and reads the profile detail modal state.
#[derive(Clone, Copy)]
pub struct ProfileModalContext {
    pub open: Signal<bool>,
    pub account: Signal<Option<AccountData>>,
    pub set_open: WriteSignal<bool>,
    pub set_account: WriteSignal<Option<AccountData>>,
}

impl ProfileModalContext {
    /// Provide a default closed profile modal context.
    pub fn provide_with_default() {
        let (open, set_open) = signal(false);
        let (account, set_account) = signal::<Option<AccountData>>(None);
        provide_context(Self {
            open: open.into(),
            account: account.into(),
            set_open,
            set_account,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }

    /// Open the modal for the given account.
    pub fn open(&self, account: AccountData) {
        self.set_account.set(Some(account));
        self.set_open.set(true);
    }

    /// Close the modal and clear the selected account.
    pub fn close(&self) {
        self.set_open.set(false);
        self.set_account.set(None);
    }
}
