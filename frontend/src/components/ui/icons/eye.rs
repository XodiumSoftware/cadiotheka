use leptos::prelude::*;

/// Renders an open eye, shown when a secret is visible and clicking hides it.
#[component]
pub fn EyeIcon(
    /// Size classes for the SVG; defaults to a 0.875rem square.
    #[prop(default = "w-3.5 h-3.5")]
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
            <path d="M2 12s3-7 10-7 10 7 10 7-3 7-10 7-10-7-10-7Z" />
            <circle cx="12" cy="12" r="3" />
        </svg>
    }
}
