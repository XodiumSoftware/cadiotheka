use leptos::prelude::*;

/// Renders the clock glyph used next to the relative publication time.
///
/// Strokes with `currentColor`; size it through the `class` prop, which defaults
/// to the size the project card used to hard-code.
#[component]
pub fn ClockIcon(
    /// Size classes for the SVG; defaults to a 0.875rem square.
    #[prop(default = "w-3.5 h-3.5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <svg
            class=class
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <circle cx="12" cy="12" r="10" />
            <path d="M12 6v6l4 2" />
        </svg>
    }
}
