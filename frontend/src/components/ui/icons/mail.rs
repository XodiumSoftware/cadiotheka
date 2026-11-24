use leptos::prelude::*;

/// Renders the mail envelope used beside the account email.
#[component]
pub fn MailIcon(
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
            <rect x="2" y="4" width="20" height="16" rx="2" />
            <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
        </svg>
    }
}
