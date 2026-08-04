use leptos::prelude::*;

/// Item rendered inside an [`OverflowRow`].
#[derive(Clone)]
pub struct OverflowItem {
    pub label: String,
    /// Inline CSS style string for the item's color, e.g. `"background-color:#1d4ed8;color:#ffffff"`.
    ///
    /// Stored colors are applied directly via the `style` attribute so they work
    /// regardless of Tailwind's build-time class scanning.
    pub color_style: String,
}

impl OverflowItem {
    pub fn new(label: impl Into<String>, color_style: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color_style: color_style.into(),
        }
    }
}

/// Renders a row of badge-like items with overflow collapsed into a "+N" box.
///
/// The container is `flex-nowrap` so it never wraps. Items beyond `max_visible`
/// are hidden and summarized by the overflow box, which shows a DaisyUI tooltip
/// listing all hidden labels vertically.
#[component]
pub fn OverflowRow(
    #[prop(into)] items: Vec<OverflowItem>,
    #[prop(default = 3)] max_visible: usize,
    #[prop(optional)] tooltip_position: Option<&'static str>,
    badge_class: &'static str,
) -> impl IntoView {
    let mut items = items;
    let hidden = items.split_off(items.len().min(max_visible));
    let visible = items;
    let overflow_count = hidden.len();
    let tooltip = hidden
        .iter()
        .map(|item| item.label.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    view! {
        <div class="flex flex-nowrap items-center gap-1 overflow-hidden">
            {visible
                .into_iter()
                .map(|item| {
                    let class = format!("{badge_class} whitespace-nowrap");
                    let style = item.color_style;
                    view! {
                        <span class=class style=style>{item.label}</span>
                    }
                })
                .collect_view()}
            {if overflow_count > 0 {
                let tooltip_class = format!(
                    "tooltip {} badge badge-xs badge-outline rounded-none border-base-content/20 text-base-content/70 cursor-help flex-shrink-0",
                    tooltip_position.unwrap_or("tooltip-bottom")
                );
                view! {
                    <span
                        class=tooltip_class
                        data-tip={tooltip}
                    >
                        {format!("+{overflow_count}")}
                    </span>
                }
                    .into_any()
            } else {
                ().into_any()
            }}
        </div>
    }
}
