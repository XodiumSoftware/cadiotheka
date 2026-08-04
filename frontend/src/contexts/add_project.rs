use leptos::prelude::*;

/// Provides and reads the add-project modal state.
#[derive(Clone, Copy)]
pub struct AddProjectModalContext {
    pub open: Signal<bool>,
    pub set_open: WriteSignal<bool>,
}

impl AddProjectModalContext {
    /// Provide a default closed add-project modal context.
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

    /// Open the modal.
    pub fn open(&self) {
        self.set_open.set(true);
    }

    /// Close the modal.
    pub fn close(&self) {
        self.set_open.set(false);
    }
}
