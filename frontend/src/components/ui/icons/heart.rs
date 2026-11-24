use leptos::prelude::*;

/// Renders the heart glyph used for the favorite count.
///
/// Strokes with `currentColor`; size it through the `class` prop, which defaults
/// to the size the project card used to hard-code.
#[component]
pub fn HeartIcon(
    /// When true the heart is filled with the current color; otherwise it is
    /// only outlined.
    #[prop(optional, into)]
    filled: Signal<bool>,
    /// Size classes for the SVG; defaults to a 0.875rem square.
    #[prop(default = "w-3.5 h-3.5")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <svg
            class=class
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill=move || if filled.get() { "currentColor" } else { "none" }
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
        </svg>
    }
}
