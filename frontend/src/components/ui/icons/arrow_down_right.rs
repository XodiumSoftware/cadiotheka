use leptos::prelude::*;

/// Renders the Phosphor diagonal arrow pointing down-right.
///
/// Painted with `currentColor` and hidden from assistive technology, so the
/// button or toolbar that wraps it carries the accessible name.
#[component]
pub fn ArrowDownRightIcon(
    /// Size classes for the wrapper span; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center {class}") aria-hidden="true">
            <svg viewBox="0 0 256 256" fill="currentColor">
                <path d="M200,88V192a8,8,0,0,1-8,8H88a8,8,0,0,1,0-16h84.69L58.34,69.66A8,8,0,0,1,69.66,58.34L184,172.69V88a8,8,0,0,1,16,0Z"/>
            </svg>
        </span>
    }
}
