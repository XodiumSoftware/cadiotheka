use leptos::prelude::*;

/// Renders the log-in arrow with its door, used by the sign-in button.
///
/// Strokes with `currentColor` and is hidden from assistive technology, so the
/// button that wraps it carries the accessible name.
#[component]
pub fn SignInIcon(
    /// Size classes for the SVG; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />
            <polyline points="10 17 15 12 10 7" />
            <line x1="15" y1="12" x2="3" y2="12" />
        </svg>
    }
}
