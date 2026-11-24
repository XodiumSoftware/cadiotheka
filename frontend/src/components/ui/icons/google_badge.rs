use leptos::prelude::*;

/// Renders a circled "G" badge standing in for a linked Google account.
#[component]
pub fn GoogleBadgeIcon(
    /// Size classes for the SVG; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <svg class=class viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
            <circle cx="12" cy="12" r="10"/>
            <text x="12" y="16" text-anchor="middle" font-size="12" fill="currentColor" stroke="none">"G"</text>
        </svg>
    }
}
