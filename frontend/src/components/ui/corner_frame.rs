use leptos::prelude::*;

/// Draws decorative box-drawing corner glyphs around its children.
///
/// The four corners are absolutely positioned spans outside the content box, so
/// the wrapper is `relative` and the children fill the remaining space.
#[component]
pub fn CornerFrame(
    children: Children,
    /// Corner glyph family: `"square"`, `"curly"`, or any other value for the
    /// default angle brackets.
    #[prop(default = "angle")]
    style: &'static str,
    /// Extra classes applied to the relative wrapper element.
    #[prop(default = "")]
    class: &'static str,
    /// Renders the corner glyphs in solid black instead of the translucent
    /// primary theme color.
    #[prop(default = false)]
    black: bool,
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
        <div class={format!("relative {class}")}>
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
