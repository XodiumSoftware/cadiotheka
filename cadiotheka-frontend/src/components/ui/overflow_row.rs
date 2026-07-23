use leptos::prelude::*;

/// Item rendered inside an [`OverflowRow`].
#[derive(Clone)]
pub struct OverflowItem {
    pub label: String,
    pub color_class: String,
}

impl OverflowItem {
    pub fn new(label: impl Into<String>, color_class: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color_class: color_class.into(),
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
    #[prop(into)] badge_class: String,
) -> impl IntoView {
    let total = items.len();
    let visible: Vec<OverflowItem> = items.iter().take(max_visible).cloned().collect();
    let hidden: Vec<OverflowItem> = items.iter().skip(max_visible).cloned().collect();
    let overflow_count = total.saturating_sub(max_visible);
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
                    let class = format!("{} {} whitespace-nowrap", badge_class, item.color_class);
                    view! {
                        <span class=class>{item.label}</span>
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
