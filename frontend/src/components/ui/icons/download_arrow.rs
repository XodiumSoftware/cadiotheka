use leptos::prelude::*;

/// Renders the download-arrow glyph used next to the download count.
///
/// Strokes with `currentColor`; size it through the `class` prop, which defaults
/// to the size the project card used to hard-code.
#[component]
pub fn DownloadArrowIcon(
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
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
    }
}
