use crate::components::cards::project_card::ProjectCardProperties;
use leptos::prelude::*;

/// Provides and reads the project detail modal state.
#[derive(Clone, Copy)]
pub struct ProjectModalContext {
    pub open: Signal<bool>,
    pub card: Signal<Option<ProjectCardProperties>>,
    pub set_open: WriteSignal<bool>,
    pub set_card: WriteSignal<Option<ProjectCardProperties>>,
}

impl ProjectModalContext {
    /// Provide a default closed project modal context.
    pub fn provide_with_default() {
        let (open, set_open) = signal(false);
        let (card, set_card) = signal::<Option<ProjectCardProperties>>(None);
        provide_context(Self {
            open: open.into(),
            card: card.into(),
            set_open,
            set_card,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }

    /// Open the modal with the given card properties.
    pub fn open(&self, card: ProjectCardProperties) {
        self.set_card.set(Some(card));
        self.set_open.set(true);
    }

    /// Close the modal and clear the selected card.
    pub fn close(&self) {
        self.set_open.set(false);
        self.set_card.set(None);
    }
}
