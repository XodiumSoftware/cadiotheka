use leptos::prelude::*;

/// Renders the Phosphor diagonal arrow pointing down-left.
///
/// Painted with `currentColor` and hidden from assistive technology, so the
/// button or toolbar that wraps it carries the accessible name.
#[component]
pub fn ArrowDownLeftIcon(
    /// Size classes for the wrapper span; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center {class}") aria-hidden="true">
            <svg viewBox="0 0 256 256" fill="currentColor">
                <path d="M197.66,69.66,83.31,184H168a8,8,0,0,1,0,16H64a8,8,0,0,1-8-8V88a8,8,0,0,1,16,0v84.69L186.34,58.34a8,8,0,0,1,11.32,11.32Z"/>
            </svg>
        </span>
    }
}
