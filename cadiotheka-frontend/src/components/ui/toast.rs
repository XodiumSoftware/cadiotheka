use leptos::prelude::*;

/// A brief, auto-dismissible toast notification fixed to the top-center of the
/// viewport.
#[component]
pub fn Toast(
    #[prop(into)] message: Signal<String>,
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] on_dismiss: Callback<()>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if visible.get() {
                    "fixed top-4 left-1/2 -translate-x-1/2 z-[100] px-4 py-2 bg-primary text-primary-content text-sm font-medium shadow-lg border border-primary transition-opacity duration-200 opacity-100 cursor-pointer"
                } else {
                    "fixed top-4 left-1/2 -translate-x-1/2 z-[100] px-4 py-2 bg-primary text-primary-content text-sm font-medium shadow-lg border border-primary transition-opacity duration-200 opacity-0 pointer-events-none"
                }
            }
            role="status"
            aria-live="polite"
            aria-hidden=move || !visible.get()
            on:click=move |_| on_dismiss.run(())
        >
            {message}
        </div>
    }
}
