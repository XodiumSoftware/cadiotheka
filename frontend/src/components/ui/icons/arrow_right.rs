use leptos::prelude::*;

/// Renders the Phosphor arrow pointing right.
///
/// Painted with `currentColor` and hidden from assistive technology, so the
/// button or toolbar that wraps it carries the accessible name.
#[component]
pub fn ArrowRightIcon(
    /// Size classes for the wrapper span; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center {class}") aria-hidden="true">
            <svg viewBox="0 0 256 256" fill="currentColor">
                <path d="M181.66,133.66l-80,80a8,8,0,0,1-11.32-11.32L164.69,128,90.34,53.66a8,8,0,0,1,11.32-11.32l80,80A8,8,0,0,1,181.66,133.66Z"/>
            </svg>
        </span>
    }
}
