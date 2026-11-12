use crate::data::AccountData;
use leptos::prelude::*;

/// Provides and reads the profile detail modal state.
#[derive(Clone, Copy)]
pub struct ProfileModalContext {
    /// Whether the profile detail modal is currently visible.
    pub open: Signal<bool>,
    /// The account whose profile is being displayed.
    pub account: Signal<Option<AccountData>>,
    /// Setter that opens or closes the modal directly.
    pub set_open: WriteSignal<bool>,
    /// Setter that swaps the account shown in the modal.
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
        expect_context::<Self>()
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
