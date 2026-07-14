use crate::components::ui::cornerframe::CornerFrame;
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
            <div class="w-full max-w-lg">
                <div class="block p-2 bg-base-100 border-2 border-primary">
                    <CornerFrame style="square" class="w-full">
                        <div class="h-full rounded-none p-6">
                            {children_view}
                        </div>
                    </CornerFrame>
                </div>
            </div>
        </div>
    }
}
