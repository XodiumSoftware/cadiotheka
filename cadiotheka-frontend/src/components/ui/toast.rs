use leptos::prelude::*;

/// Builds the toast DOM for the given state.
fn toast_view(
    visible: Signal<bool>,
    message: Signal<String>,
    on_dismiss: Callback<()>,
) -> impl IntoView {
    view! {
        <div
            class=move || {
                if visible.get() {
                    "fixed top-4 left-1/2 -translate-x-1/2 z-[9999] flex flex-col gap-2 transition-opacity duration-200 opacity-100"
                } else {
                    "fixed top-4 left-1/2 -translate-x-1/2 z-[9999] flex flex-col gap-2 transition-opacity duration-200 opacity-0 pointer-events-none"
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

/// A brief, auto-dismissible toast notification fixed to the top-center of the
/// viewport using DaisyUI toast styling. It is rendered as a portal to the
/// document body so it always sits above modals and backdrops.
#[component]
pub fn Toast(
    #[prop(into)] message: Signal<String>,
    #[prop(into)] visible: Signal<bool>,
    #[prop(into)] on_dismiss: Callback<()>,
) -> impl IntoView {
    let body = leptos::web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.body());

    match body {
        Some(body) => view! {
            <leptos::portal::Portal mount=body>
                {toast_view(visible, message, on_dismiss)}
            </leptos::portal::Portal>
        }
        .into_any(),
        None => toast_view(visible, message, on_dismiss).into_any(),
    }
}
