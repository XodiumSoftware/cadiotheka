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
