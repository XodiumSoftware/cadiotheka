use leptos::prelude::*;

/// Renders the magnifier glyph for the search button.
///
/// Strokes with `currentColor`; size it through the `class` prop, which defaults
/// to the size the header used to hard-code.
#[component]
pub fn SearchIcon(
    /// Size classes for the SVG; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
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
            <circle cx="11" cy="11" r="8" />
            <path d="m21 21-4.3-4.3" />
        </svg>
    }
}
