use leptos::prelude::*;

/// Renders the file-with-plus glyph standing in for an IFC document.
///
/// Its colour is set by the caller through `class`, since the version state
/// tints the glyph. The class is a `String` so callers can append that tint.
#[component]
pub fn FilePlusIcon(
    /// Size classes for the SVG; defaults to a w-5 h-5 square.
    #[prop(default = "w-5 h-5".to_string())]
    class: String,
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
            <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
            <polyline points="14 2 14 8 20 8" />
            <line x1="12" y1="18" x2="12" y2="12" />
            <line x1="9" y1="15" x2="15" y2="15" />
        </svg>
    }
}
