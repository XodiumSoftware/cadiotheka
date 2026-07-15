use crate::i18n::{t_string, use_i18n};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

/// A Flowbite-style rounded toggle switch with optional labels.
#[component]
pub fn ToggleSlider(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(optional)] label_left: Option<&'static str>,
    #[prop(optional)] label_right: Option<&'static str>,
) -> impl IntoView {
    let input_id = "layout-toggle";

    view! {
        <label
            for=input_id
            class="inline-flex items-center cursor-pointer select-none gap-2"
        >
            {label_left.map(|label| {
                view! { <span class="text-sm font-medium text-base-content/80">{label}</span> }
            })}

            <input
                id=input_id
                type="checkbox"
                class="sr-only peer"
                prop:checked=move || checked.get()
                on:change=move |ev| {
                    if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                        on_change.run(input.checked());
                    }
                }
            />

            <div class="relative w-11 h-6 bg-base-300 rounded-full peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-base-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>

            {label_right.map(|label| {
                view! { <span class="text-sm font-medium text-base-content/80">{label}</span> }
            })}
        </label>
    }
}

/// A toggle switch that shows a combined "Off / On" label before the slider.
/// The active side is emphasized based on `checked`.
/// An optional keyboard shortcut hint can be appended after the slider.
#[component]
pub fn ToggleSliderWithSlashLabel(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(into)] label_left: Signal<String>,
    #[prop(into)] label_right: Signal<String>,
    #[prop(optional)] shortcut_hint: Option<Signal<String>>,
) -> impl IntoView {
    let i18n = use_i18n();
    let input_id = "layout-toggle-slash";

    view! {
        <label
            for=input_id
            class="inline-flex items-center cursor-pointer select-none gap-2"
        >
            <span class="text-sm font-medium text-base-content/60">
                <span class=move || if checked.get() { "text-base-content/40" } else { "text-base-content" }>{move || label_left.get()}</span>
                <span class="mx-0.5">/</span>
                <span class=move || if checked.get() { "text-base-content" } else { "text-base-content/40" }>{move || label_right.get()}</span>
            </span>

            <input
                id=input_id
                type="checkbox"
                class="sr-only peer"
                prop:checked=move || checked.get()
                on:change=move |ev| {
                    if let Some(input) = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) {
                        on_change.run(input.checked());
                    }
                }
            />

            <div class="relative w-11 h-6 bg-base-300 rounded-full peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-primary/50 peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-base-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-primary"></div>

            {shortcut_hint.map(|hint| {
                view! {
                    <span class="mx-1 text-base-content/50">{move || t_string!(i18n, search.hint_or)}</span>
                    <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{move || hint.get()}</kbd>
                }
            })}
        </label>
    }
}
