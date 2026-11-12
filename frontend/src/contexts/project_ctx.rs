use crate::components::cards::project::ProjectCardProperties;
use crate::components::ui::modals::project::ProjectDetailsTab;
use leptos::prelude::*;

/// Provides and reads the project detail modal state.
#[derive(Clone, Copy)]
pub struct ProjectModalContext {
    /// Whether the project detail modal is currently visible.
    pub open: Signal<bool>,
    /// The project card whose details are being shown.
    pub card: Signal<Option<ProjectCardProperties>>,
    /// The details tab selected in the modal, defaulting to the 3D viewer.
    pub active_tab: Signal<ProjectDetailsTab>,
    /// Setter that opens or closes the modal directly.
    pub set_open: WriteSignal<bool>,
    /// Setter that swaps the project shown in the modal.
    pub set_card: WriteSignal<Option<ProjectCardProperties>>,
    /// Setter that switches the active details tab.
    pub set_active_tab: WriteSignal<ProjectDetailsTab>,
}

impl ProjectModalContext {
    /// Provide a default closed project modal context.
    pub fn provide_with_default() {
        let (open, set_open) = signal(false);
        let (card, set_card) = signal::<Option<ProjectCardProperties>>(None);
        let (active_tab, set_active_tab) = signal(ProjectDetailsTab::Viewer3d);
        provide_context(Self {
            open: open.into(),
            card: card.into(),
            active_tab: active_tab.into(),
            set_open,
            set_card,
            set_active_tab,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        expect_context::<Self>()
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
