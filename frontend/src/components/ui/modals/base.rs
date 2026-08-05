//! Base modal using the native HTML `dialog` element.
//!
//! This follows the `DaisyUI` recommended approach: a `dialog class="modal"`
//! containing a `div class="modal-box"` for content and a `form
//! method="dialog" class="modal-backdrop"` to close when clicking outside.
//! Native `dialog` handles `Esc` to close, backdrop focus trapping, and
//! scrollbar management automatically.

use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

/// A reusable modal dialog built on the native HTML `dialog` element.
#[component]
pub fn BaseModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into, default = Callback::new(|_| {}))] on_inner_click: Callback<()>,
    #[prop(into, default = Signal::derive(|| "w-full max-w-lg max-h-[80vh] flex flex-col".to_string()))]
    container_class: Signal<String>,
    children: Children,
) -> impl IntoView {
    let dialog_ref: NodeRef<leptos::html::Dialog> = NodeRef::new();

    // Keep the dialog's native open/closed state in sync with the signal.
    Effect::new(move |_| {
        let Some(dialog) = dialog_ref.get() else {
            return;
        };
        if open.get() {
            if !dialog.open() {
                let _ = dialog.show_modal();
            }
        } else if dialog.open() {
            dialog.close();
        }
    });

    let children_view = children();

    view! {
        <dialog
            node_ref=dialog_ref
            class="modal"
            role="dialog"
            aria-modal="true"
            on:close=move |_| on_close.run(())
            on:click=move |ev: web_sys::MouseEvent| {
                if let Some(target) = ev.target()
                    && let Ok(clicked) = target.dyn_into::<web_sys::Node>()
                    && let Some(dialog) = dialog_ref.get()
                    && dialog.is_same_node(Some(&clicked))
                {
                    on_close.run(());
                } else {
                    on_inner_click.run(());
                }
            }
        >
            <div
                class=move || {
                    format!(
                        "modal-box rounded-none p-0 overflow-hidden bg-base-100 border-2 border-primary {}",
                        container_class.get()
                    )
                }
                role="dialog"
                aria-modal="true"
                on:click=move |_| on_inner_click.run(())
            >
                <div class="h-full p-6 flex flex-col">
                    {children_view}
                </div>
            </div>
            <form method="dialog" class="modal-backdrop">
                <button type="submit">"close"</button>
            </form>
        </dialog>
    }
}
