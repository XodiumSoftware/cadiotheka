use leptos::prelude::*;

/// Simple fade overlay for any section
#[component]
pub fn FadeOverlay(
    #[prop(default = "")] position: &'static str,
    /// Height of the fade in tailwind spacing
    #[prop(default = "20")]
    height: &'static str,
) -> impl IntoView {
    match position {
        "top" => view! {
            <div class={format!("absolute inset-x-0 top-0 h-{} bg-gradient-to-b from-base-100 to-transparent pointer-events-none", height)} />
        }.into_any(),
        "bottom" => view! {
            <div class={format!("absolute inset-x-0 bottom-0 h-{} bg-gradient-to-t from-base-100 to-transparent pointer-events-none", height)} />
        }.into_any(),
        _ => view! {
            <>
                <div class={format!("absolute inset-x-0 top-0 h-{} bg-gradient-to-b from-base-100 to-transparent pointer-events-none", height)} />
                <div class={format!("absolute inset-x-0 bottom-0 h-{} bg-gradient-to-t from-base-100 to-transparent pointer-events-none", height)} />
            </>
        }.into_any(),
    }
}
