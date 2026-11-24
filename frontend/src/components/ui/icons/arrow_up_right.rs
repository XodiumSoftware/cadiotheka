use leptos::prelude::*;

/// Renders the Phosphor diagonal arrow pointing up-right.
///
/// Painted with `currentColor` and hidden from assistive technology, so the
/// button or toolbar that wraps it carries the accessible name.
#[component]
pub fn ArrowUpRightIcon(
    /// Size classes for the wrapper span; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center {class}") aria-hidden="true">
            <svg viewBox="0 0 256 256" fill="currentColor">
                <path d="M200,64V168a8,8,0,0,1-16,0V83.31L69.66,197.66a8,8,0,0,1-11.32-11.32L172.69,72H88a8,8,0,0,1,0-16H192A8,8,0,0,1,200,64Z"/>
            </svg>
        </span>
    }
}
