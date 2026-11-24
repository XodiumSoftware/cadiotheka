use leptos::prelude::*;

/// Renders the minus sign used for "remove" and "disconnect" overlays.
///
/// Strokes with `currentColor` and is hidden from assistive technology, so the
/// button that wraps it carries the accessible name.
#[component]
pub fn MinusIcon(
    /// Size classes for the SVG; defaults to a 1.25rem square.
    #[prop(default = "w-5 h-5")]
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
            <line x1="5" y1="12" x2="19" y2="12" />
        </svg>
    }
}
