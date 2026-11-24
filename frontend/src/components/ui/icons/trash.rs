use leptos::prelude::*;

/// Renders the trash can used for delete actions.
///
/// Hidden from assistive technology; the button that wraps it carries the
/// accessible name.
#[component]
pub fn TrashIcon(
    /// Size classes for the SVG; defaults to a w-4 h-4 square.
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
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
        </svg>
    }
}
