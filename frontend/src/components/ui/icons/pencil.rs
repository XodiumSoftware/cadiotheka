use leptos::prelude::*;

/// SVG markup for the pencil, for callers that inject it as `inner_html`.
///
/// [`IconToggle`](crate::components::ui::toggle::IconToggle) swaps its two states
/// through `inner_html`, so it cannot take a component. This constant keeps that
/// path on the same glyph as [`PencilIcon`] instead of a private copy.
pub const PENCIL_MARKUP: &str = r#"<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="w-5 h-5"><path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z"/></svg>"#;

/// Renders the pencil used for edit affordances and hover overlays.
#[component]
pub fn PencilIcon(
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
            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
        </svg>
    }
}
