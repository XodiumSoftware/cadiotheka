use leptos::prelude::*;

#[component]
pub fn CornerFrame(
    children: Children,
    #[prop(default = "angle")] style: &'static str,
    #[prop(default = "")] class: &'static str,
    #[prop(default = false)] black: bool,
) -> impl IntoView {
    let (top_left, top_right, bottom_left, bottom_right) = match style {
        "square" => ("┌", "┐", "└", "┘"),
        "curly" => ("╭", "╮", "╰", "╯"),
        _ => ("⟨", "⟩", "⟨", "⟩"), // angle brackets (default)
    };

    let corner_class = if black {
        "absolute text-black font-mono text-sm leading-none pointer-events-none select-none"
    } else {
        "absolute text-primary/40 font-mono text-sm leading-none pointer-events-none select-none"
    };

    view! {
        <div class={format!("relative {}", class)}>
            <span class={format!("{corner_class} -top-2 -left-1")}>
                {top_left}
            </span>
            <span class={format!("{corner_class} -top-2 -right-1")}>
                {top_right}
            </span>
            <span class={format!("{corner_class} -bottom-2 -left-1")}>
                {bottom_left}
            </span>
            <span class={format!("{corner_class} -bottom-2 -right-1")}>
                {bottom_right}
            </span>
            {children()}
        </div>
    }
}
