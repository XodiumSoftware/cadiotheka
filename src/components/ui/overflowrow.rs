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

/// Renders a row of badge-like items, collapsing overflow into a "+N" box.
///
/// `max_visible` controls how many items are shown before the overflow box
/// appears. The overflow box shows a tooltip listing all hidden labels.
#[component]
pub fn OverflowRow(
    #[prop(into)] items: Vec<OverflowItem>,
    #[prop(default = 3)] max_visible: usize,
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
        .join(", ");

    view! {
        {visible
            .into_iter()
            .map(|item| {
                let class = format!("{} {}", badge_class, item.color_class);
                view! {
                    <span class=class>{item.label}</span>
                }
            })
            .collect_view()}
        {if overflow_count > 0 {
            view! {
                <span
                    class="badge badge-xs badge-outline rounded-none border-base-content/20 text-base-content/70 cursor-help"
                    title={tooltip}
                >
                    {format!("+{}", overflow_count)}
                </span>
            }
                .into_any()
        } else {
            ().into_any()
        }}
    }
}
