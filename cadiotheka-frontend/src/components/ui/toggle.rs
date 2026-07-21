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
            class="hidden sm:inline-flex items-center cursor-pointer select-none gap-2"
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

/// DaisyUI `swap` toggle that swaps between two SVG icon strings.
/// `icon_off` is shown when unchecked, `icon_on` when checked.
#[component]
pub fn IconToggle(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
    #[prop(into)] icon_off: String,
    #[prop(into)] icon_on: String,
    #[prop(optional)] tooltip_off: Option<&'static str>,
    #[prop(optional)] tooltip_on: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let aria_label = move || {
        if checked.get() {
            tooltip_on.unwrap_or("Toggle off")
        } else {
            tooltip_off.unwrap_or("Toggle on")
        }
    };

    view! {
        <button
            type="button"
            class=move || {
                format!(
                    "swap swap-rotate cursor-pointer inline-flex items-center justify-center {} tooltip tooltip-bottom {}",
                    class.unwrap_or(""),
                    if checked.get() { "swap-active" } else { "" }
                )
            }
            data-tip=move || {
                if checked.get() {
                    tooltip_on.unwrap_or("")
                } else {
                    tooltip_off.unwrap_or("")
                }
            }
            aria-label=aria_label
            on:click=move |_| on_change.run(!checked.get_untracked())
        >
            <span
                class="swap-off text-base-content/50 w-full h-full flex items-center justify-center"
                inner_html=icon_off
            ></span>
            <span
                class="swap-on text-primary w-full h-full flex items-center justify-center"
                inner_html=icon_on
            ></span>
        </button>
    }
}

/// A pencil-icon edit toggle. The pencil is muted when edit is off and primary when edit is on.
#[component]
pub fn EditToggle(
    #[prop(into)] checked: Signal<bool>,
    #[prop(into)] on_change: Callback<bool>,
) -> impl IntoView {
    let pencil_icon = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-5 h-5"><path d="M17 3a2.828 2.828 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5L17 3z"/></svg>"#;

    view! {
        <IconToggle
            checked=checked
            on_change=on_change
            icon_off=pencil_icon.to_string()
            icon_on=pencil_icon.to_string()
            tooltip_off="Enter edit mode"
            tooltip_on="Exit edit mode"
            class="w-8 h-8 rounded hover:bg-base-200 transition-colors"
        />
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
    let input_id = "layout-toggle-slash";

    view! {
        <label
            for=input_id
            class="hidden sm:inline-flex items-center cursor-pointer select-none gap-2"
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
                    <span class="mx-1 text-base-content/50 hidden sm:inline">{"or"}</span>
                    <kbd class="hidden sm:inline-flex px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{move || hint.get()}</kbd>
                }
            })}
        </label>
    }
}
