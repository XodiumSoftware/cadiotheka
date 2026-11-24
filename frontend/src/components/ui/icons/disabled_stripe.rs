use leptos::prelude::*;

/// Renders the diagonal error-colored stripe laid over an unavailable tool.
///
/// It stretches to fill a positioned parent and ignores pointer events, so the
/// button underneath still receives clicks. Hidden from assistive technology.
#[component]
pub fn DisabledStripeIcon() -> impl IntoView {
    view! {
        <svg
            class="absolute inset-0 w-full h-full text-error pointer-events-none"
            viewBox="0 0 16 16"
            fill="none"
            aria-hidden="true"
        >
            <path d="M2 14 L14 2" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
        </svg>
    }
}
