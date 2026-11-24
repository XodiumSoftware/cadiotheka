use leptos::prelude::*;

/// Renders the left-pointing chevron used by the pagination arrows.
///
/// Strokes with `currentColor` and is hidden from assistive technology, so the
/// button that wraps it carries the accessible name.
#[component]
pub fn ChevronLeftIcon(
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
            <polyline points="15 18 9 12 15 6" />
        </svg>
    }
}
