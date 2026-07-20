use crate::components::ui::modals::search::SearchModal;
use crate::contexts::AddProjectModalContext;
use crate::i18n::{t_string, use_i18n};
use leptos::prelude::*;

/// Stub modal for adding a new project.
///
/// This currently shows a placeholder message. The actual form will be added
/// once the backend API supports project creation from the frontend.
#[component]
pub fn AddProjectModal() -> impl IntoView {
    let i18n = use_i18n();
    let modal = AddProjectModalContext::use_context();
    let on_close = move |_| modal.close();
    let on_button_click = move |_| modal.close();

    view! {
        <SearchModal
            open=modal.open
            on_close=on_close
        >
            <div class="space-y-6 flex flex-col min-h-0">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-bold text-primary">{move || t_string!(i18n, add_project.title)}</h2>
                    <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50">
                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            {move || t_string!(i18n, search.keyboard_esc)}
                        </kbd>
                        <span>{move || t_string!(i18n, project_modal.hint_dismiss)}</span>
                    </div>
                </div>

                <p class="text-sm text-base-content/80">
                    {move || t_string!(i18n, add_project.stub_message)}
                </p>

                <div class="flex justify-end">
                    <button
                        type="button"
                        class="btn btn-primary btn-lift"
                        on:click=on_button_click
                    >
                        {move || t_string!(i18n, add_project.close)}
                    </button>
                </div>
            </div>
        </SearchModal>
    }
}
