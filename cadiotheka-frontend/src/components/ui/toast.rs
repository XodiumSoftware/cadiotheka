use leptos::prelude::*;

/// A brief, auto-dismissible toast notification fixed to the top-center of the
/// viewport using DaisyUI toast styling.
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
                    "toast toast-top toast-center z-[100] transition-opacity duration-200 opacity-100"
                } else {
                    "toast toast-top toast-center z-[100] transition-opacity duration-200 opacity-0 pointer-events-none"
                }
            }
            aria-hidden=move || !visible.get()
        >
            <div
                class="alert alert-primary shadow-lg cursor-pointer"
                role="status"
                aria-live="polite"
                on:click=move |_| on_dismiss.run(())
            >
                <span class="font-bold text-primary-content">{message}</span>
            </div>
        </div>
    }
}
