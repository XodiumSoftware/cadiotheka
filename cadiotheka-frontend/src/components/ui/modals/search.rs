use crate::components::ui::corner_frame::CornerFrame;
use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

/// A reusable modal dialog with a search-modal visual style.
#[component]
pub fn SearchModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into, default = Callback::new(|_| {}))] on_inner_click: Callback<()>,
    children: Children,
) -> impl IntoView {
    let children_view = children();
    let backdrop_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    // Close the modal when the user presses Escape. The listener is only
    // registered while the modal is open and removed immediately when it closes.
    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", {
            let on_close = on_close;
            move |ev| {
                if ev.key().eq_ignore_ascii_case("escape") {
                    ev.prevent_default();
                    on_close.run(());
                }
            }
        });
    });

    Effect::new(move |_| {
        if let Some(body) = leptos::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            if open.get() {
                body.class_list().add_1("overflow-hidden").ok();
            } else {
                body.class_list().remove_1("overflow-hidden").ok();
            }
        }
    });

    // Ensure the body scroll lock is released if the modal is unmounted while
    // still open.
    leptos::prelude::on_cleanup(move || {
        if let Some(body) = leptos::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            body.class_list().remove_1("overflow-hidden").ok();
        }
    });

    view! {
        <div
            node_ref=backdrop_ref
            class=move || {
                if open.get() {
                    "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
                } else {
                    "hidden"
                }
            }
            on:click=move |ev: leptos::web_sys::MouseEvent| {
                if let Some(target) = ev.target()
                    && let Ok(clicked) = target.dyn_into::<leptos::web_sys::Node>()
                    && let Some(backdrop) = backdrop_ref.get()
                {
                    if backdrop.is_same_node(Some(&clicked)) {
                        on_close.run(());
                    } else {
                        on_inner_click.run(());
                    }
                }
            }
        >
            <div class="w-full max-w-lg max-h-[80vh] flex flex-col">
                <div class="block p-2 bg-base-100 border-2 border-primary overflow-hidden">
                    <CornerFrame style="square" class="w-full">
                        <div class="h-full rounded-none p-6 overflow-hidden flex flex-col">
                            {children_view}
                        </div>
                    </CornerFrame>
                </div>
            </div>
        </div>
    }
}
