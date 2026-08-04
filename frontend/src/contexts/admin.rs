use leptos::prelude::*;

/// Provides and reads the admin metadata management modal state.
#[derive(Clone, Copy)]
pub struct AdminModalContext {
    pub open: Signal<bool>,
    pub set_open: WriteSignal<bool>,
}

impl AdminModalContext {
    /// Provide a default closed admin modal context.
    pub fn provide_with_default() {
        let (open, set_open) = signal(false);
        provide_context(Self {
            open: open.into(),
            set_open,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }

    /// Open the admin modal.
    pub fn open(&self) {
        self.set_open.set(true);
    }

    /// Close the admin modal.
    pub fn close(&self) {
        self.set_open.set(false);
    }
}
