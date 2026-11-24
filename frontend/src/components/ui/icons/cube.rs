use leptos::prelude::*;

/// Renders the Phosphor solid cube.
///
/// Painted with `currentColor` and hidden from assistive technology, so the
/// button or toolbar that wraps it carries the accessible name.
#[component]
pub fn CubeIcon(
    /// Size classes for the wrapper span; defaults to a 1rem square.
    #[prop(default = "w-4 h-4")]
    class: &'static str,
) -> impl IntoView {
    view! {
        <span class=format!("inline-flex items-center justify-center {class}") aria-hidden="true">
            <svg viewBox="0 0 256 256" fill="currentColor">
                <path d="M223.68,74.37,138.21,32a8,8,0,0,0-7.42,0L32.32,74.37A8,8,0,0,0,28,81.39v87.22a8,8,0,0,0,4.32,7l85.47,42.74a8,8,0,0,0,7.42,0l85.47-42.74a8,8,0,0,0,4.32-7V81.39A8,8,0,0,0,223.68,74.37ZM128,42.23l74.29,37.15L128,116.52,53.71,79.38ZM44,92.63l72,36v70.58l-72-36Zm88,106.58V128.63l72-36v70.58Z"/>
            </svg>
        </span>
    }
}
