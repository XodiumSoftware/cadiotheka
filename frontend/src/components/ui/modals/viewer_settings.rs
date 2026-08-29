//! 3D viewer settings modal.
//!
//! Currently supports changing the object highlight color used for hovered
//! primitives in the 3D viewer. The color is persisted account-scoped in the
//! `viewer_preferences` JSON blob.

use crate::components::Icon;
use crate::components::ui::modals::base::BaseModal;
use crate::utils::{contrast_color, hex_to_srgba, srgba_to_hex};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use three_d_asset::Srgba;

const DEFAULT_FOV_DEGREES: f32 = 45.0;

/// Modal dialog for 3D viewer settings.
#[component]
pub fn ViewerSettingsModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into)] highlight_color: RwSignal<Srgba>,
    #[prop(into)] selection_color: RwSignal<Srgba>,
    #[prop(into)] skybox_color: RwSignal<Srgba>,
    #[prop(into)] show_fps: RwSignal<bool>,
    #[prop(into)] fov_degrees: RwSignal<f32>,
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

    let on_selection_input = move |ev: leptos::web_sys::Event| {
        let value = ev
            .target()
            .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default();
        if let Some(color) = hex_to_srgba(&value) {
            selection_color.set(color);
        }
    };

    let on_skybox_input = move |ev: leptos::web_sys::Event| {
        let value = ev
            .target()
            .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default();
        if let Some(color) = hex_to_srgba(&value) {
            skybox_color.set(color);
        }
    };

    let on_fps_toggle = move |ev: leptos::web_sys::Event| {
        let checked = ev
            .target()
            .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
            .is_some_and(|input| input.checked());
        show_fps.set(checked);
    };

    let on_fov_input = move |ev: leptos::web_sys::Event| {
        let value = ev
            .target()
            .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
            .and_then(|input| input.value().parse::<f32>().ok())
            .unwrap_or(45.0);
        fov_degrees.set(value.clamp(10.0, 120.0));
    };

    let reset_show_fps = move |_| show_fps.set(false);
    let reset_fov = move |_| fov_degrees.set(DEFAULT_FOV_DEGREES);
    let reset_highlight_color = move |_| highlight_color.set(Srgba::new(255, 200, 0, 255));
    let reset_selection_color = move |_| selection_color.set(Srgba::new(0, 150, 255, 255));
    let reset_skybox_color = move |_| skybox_color.set(Srgba::WHITE);

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

                <div class="rounded-none border border-base-content/10 bg-base-200/30 p-3 space-y-4">
                    <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-2">
                            <label class="text-sm font-medium text-base-content" for="show-fps">
                                "Show FPS counter"
                            </label>
                            <button
                                type="button"
                                class="text-base-content/50 hover:text-base-content p-0.5 cursor-pointer"
                                aria-label="Reset FPS counter setting"
                                on:click=reset_show_fps
                            >
                                <Icon::Reset class="w-3.5 h-3.5" />
                            </button>
                        </div>
                        <input
                            id="show-fps"
                            type="checkbox"
                            prop:checked=move || show_fps.get()
                            on:change=on_fps_toggle
                            class="toggle toggle-primary toggle-sm"
                        />
                    </div>

                    <div class="space-y-2">
                        <div class="flex items-center justify-between gap-3">
                            <div class="flex items-center gap-2">
                                <label class="text-sm font-medium text-base-content" for="fov">
                                    "Field of view"
                                </label>
                                <button
                                    type="button"
                                    class="text-base-content/50 hover:text-base-content p-0.5 cursor-pointer"
                                    aria-label="Reset field of view"
                                    on:click=reset_fov
                                >
                                    <Icon::Reset class="w-3.5 h-3.5" />
                                </button>
                            </div>
                            <span class="text-xs font-mono text-base-content/70">
                                {move || format!("{:.0}°", fov_degrees.get())}
                            </span>
                        </div>
                        <input
                            id="fov"
                            type="range"
                            min="10"
                            max="120"
                            step="1"
                            prop:value=move || fov_degrees.get().to_string()
                            on:input=on_fov_input
                            class="range range-primary range-sm w-full"
                        />
                    </div>

                    <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-2">
                            <label class="text-sm font-medium text-base-content" for="highlight-color">
                                "Object highlight color"
                            </label>
                            <button
                                type="button"
                                class="text-base-content/50 hover:text-base-content p-0.5 cursor-pointer"
                                aria-label="Reset object highlight color"
                                on:click=reset_highlight_color
                            >
                                <Icon::Reset class="w-3.5 h-3.5" />
                            </button>
                        </div>
                        <div class="relative">
                            <input
                                id="highlight-color"
                                type="color"
                                prop:value=move || srgba_to_hex(highlight_color.get())
                                on:input=on_color_input
                                class="peer h-8 w-24 cursor-pointer appearance-none border-0 bg-transparent p-0 opacity-0 absolute inset-0"
                            />
                            <button
                                type="button"
                                class="h-8 w-24 rounded-none border border-base-content/20 bg-base-100 px-2 py-1 text-xs font-bold font-mono text-base-content peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-primary"
                                style=move || format!("background-color: {}; color: {};", srgba_to_hex(highlight_color.get()), contrast_color(highlight_color.get()))
                            >
                                {move || srgba_to_hex(highlight_color.get())}
                            </button>
                        </div>
                    </div>

                    <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-2">
                            <label class="text-sm font-medium text-base-content" for="selection-color">
                                "Object selection color"
                            </label>
                            <button
                                type="button"
                                class="text-base-content/50 hover:text-base-content p-0.5 cursor-pointer"
                                aria-label="Reset object selection color"
                                on:click=reset_selection_color
                            >
                                <Icon::Reset class="w-3.5 h-3.5" />
                            </button>
                        </div>
                        <div class="relative">
                            <input
                                id="selection-color"
                                type="color"
                                prop:value=move || srgba_to_hex(selection_color.get())
                                on:input=on_selection_input
                                class="peer h-8 w-24 cursor-pointer appearance-none border-0 bg-transparent p-0 opacity-0 absolute inset-0"
                            />
                            <button
                                type="button"
                                class="h-8 w-24 rounded-none border border-base-content/20 bg-base-100 px-2 py-1 text-xs font-bold font-mono text-base-content peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-primary"
                                style=move || format!("background-color: {}; color: {};", srgba_to_hex(selection_color.get()), contrast_color(selection_color.get()))
                            >
                                {move || srgba_to_hex(selection_color.get())}
                            </button>
                        </div>
                    </div>

                    <div class="flex items-center justify-between gap-3">
                        <div class="flex items-center gap-2">
                            <label class="text-sm font-medium text-base-content" for="skybox-color">
                                "Skybox color"
                            </label>
                            <button
                                type="button"
                                class="text-base-content/50 hover:text-base-content p-0.5 cursor-pointer"
                                aria-label="Reset skybox color"
                                on:click=reset_skybox_color
                            >
                                <Icon::Reset class="w-3.5 h-3.5" />
                            </button>
                        </div>
                        <div class="relative">
                            <input
                                id="skybox-color"
                                type="color"
                                prop:value=move || srgba_to_hex(skybox_color.get())
                                on:input=on_skybox_input
                                class="peer h-8 w-24 cursor-pointer appearance-none border-0 bg-transparent p-0 opacity-0 absolute inset-0"
                            />
                            <button
                                type="button"
                                class="h-8 w-24 rounded-none border border-base-content/20 bg-base-100 px-2 py-1 text-xs font-bold font-mono text-base-content peer-focus-visible:outline peer-focus-visible:outline-2 peer-focus-visible:outline-primary"
                                style=move || format!("background-color: {}; color: {};", srgba_to_hex(skybox_color.get()), contrast_color(skybox_color.get()))
                            >
                                {move || srgba_to_hex(skybox_color.get())}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </BaseModal>
    }
}
