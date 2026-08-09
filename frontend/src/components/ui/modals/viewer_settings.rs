//! Empty 3D viewer settings modal.
//!
//! This is a placeholder for future viewer-specific settings such as camera
//! sensitivity, background color, rendering quality, and default gizmo state.

use crate::components::ui::modals::base::BaseModal;
use leptos::prelude::*;

/// Modal dialog for 3D viewer settings.
#[component]
pub fn ViewerSettingsModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <BaseModal open=open on_close=move |()| on_close.run(())>
            <div class="space-y-6 flex flex-col min-h-0">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-bold text-primary">"3D Viewer Settings"</h2>
                    <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50">
                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            "esc"
                        </kbd>
                        <span>"to dismiss"</span>
                    </div>
                </div>

                <p class="text-sm text-base-content/80">
                    "Viewer settings will appear here in a future update."
                </p>
            </div>
        </BaseModal>
    }
}
