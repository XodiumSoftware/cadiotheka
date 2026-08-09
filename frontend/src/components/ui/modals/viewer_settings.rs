//! 3D viewer settings modal.
//!
//! Currently supports changing the object highlight color used for hovered
//! primitives in the 3D viewer. The color is persisted account-scoped in the
//! `viewer_preferences` JSON blob.

use crate::components::ui::modals::base::BaseModal;
use crate::utils::{contrast_color, hex_to_srgba, srgba_to_hex};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use three_d_asset::Srgba;

/// Modal dialog for 3D viewer settings.
#[component]
pub fn ViewerSettingsModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] highlight_color: RwSignal<Srgba>,
) -> impl IntoView {
    let on_color_input = move |ev: leptos::web_sys::Event| {
        let value = ev
            .target()
            .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default();
        if let Some(color) = hex_to_srgba(&value) {
            highlight_color.set(color);
        }
    };

    view! {
        <BaseModal open=open on_close=move |()| on_close.run(())>
            <div class="space-y-6 flex flex-col min-h-0">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-bold text-primary">"3D Viewer Settings"</h2>
                    <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50">
                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            "esc"
                        </kbd>
                        <span>"to close"</span>
                    </div>
                </div>

                <div class="rounded-none border border-base-content/10 bg-base-200/30 p-3">
                    <div class="flex items-center justify-between gap-3">
                        <label class="text-sm font-medium text-base-content" for="highlight-color">
                            "Object highlight color"
                        </label>
                        <div class="relative">
                            <input
                                id="highlight-color"
                                type="color"
                                value=move || srgba_to_hex(highlight_color.get())
                                on:input=on_color_input
                                class="peer h-8 w-24 cursor-pointer appearance-none border-0 bg-transparent p-0 opacity-0 absolute inset-0"
                            />
                            <button
                                type="button"
                                class="h-8 w-24 rounded-none border border-base-content/20 bg-base-100 px-2 py-1 text-xs font-mono text-base-content peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-primary"
                                style=move || format!("background-color: {}; color: {};", srgba_to_hex(highlight_color.get()), contrast_color(highlight_color.get()))
                            >
                                {move || srgba_to_hex(highlight_color.get())}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </BaseModal>
    }
}
