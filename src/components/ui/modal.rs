use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

/// A minimal stub modal dialog.
#[component]
pub fn Modal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let children_view = children();

    view! {
        <div
            class=move || {
                if open.get() {
                    "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
                } else {
                    "hidden"
                }
            }
            on:click=move |ev: leptos::web_sys::MouseEvent| {
                if let Some(target) = ev.target()
                    && let Ok(element) = target.dyn_into::<leptos::web_sys::Element>()
                    && element.tag_name() == "DIV"
                {
                    on_close.run(());
                }
            }
        >
            <div class="bg-base-100 border border-base-content/20 rounded-lg shadow-2xl max-w-lg w-full p-6">
                {children_view}
            </div>
        </div>
    }
}
