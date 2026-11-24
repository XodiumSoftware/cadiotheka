use leptos::prelude::*;

/// Renders the X glyph used to close panels and dialogs.
///
/// Strokes with `currentColor` and is hidden from assistive technology, so the
/// button that wraps it carries the accessible name.
#[component]
pub fn CloseIcon(
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
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
    }
}
