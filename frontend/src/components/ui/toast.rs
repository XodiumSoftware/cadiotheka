use crate::contexts::ToastContext;
use leptos::prelude::*;

/// A brief, auto-dismissible toast notification fixed to the top-center of the
/// viewport.
///
/// This component reads state from the global `ToastContext`. It must be rendered
/// as a sibling of, or outside, native `dialog` elements so it is not trapped in
/// a top-layer.
#[component]
pub fn Toast() -> impl IntoView {
    let toast = ToastContext::use_context();

    view! {
        <div
            class=move || {
                if toast.visible.get() {
                    "fixed top-4 left-1/2 -translate-x-1/2 z-[100] px-4 py-2 bg-primary text-black font-bold shadow-lg border border-primary transition-opacity duration-200 opacity-100 cursor-pointer"
                } else {
                    "fixed top-4 left-1/2 -translate-x-1/2 z-[100] px-4 py-2 bg-primary text-black font-bold shadow-lg border border-primary transition-opacity duration-200 opacity-0 pointer-events-none"
                }
            }
            role="status"
            aria-live="polite"
            aria-hidden=move || !toast.visible.get()
            on:click=move |_| toast.dismiss()
        >
            {toast.message}
        </div>
    }
}
