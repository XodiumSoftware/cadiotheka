//! Shared toolbar button component used by the markdown editor and IFC viewer.

use leptos::prelude::*;

/// Vertical tooltip placement for a toolbar button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TooltipPosition {
    /// Tooltip appears above the button.
    #[default]
    Top,
    /// Tooltip appears below the button.
    Bottom,
}

/// A small icon-style button with a configurable tooltip.
#[component]
pub fn ToolbarButton(
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    #[prop(default = TooltipPosition::Top)] tooltip_position: TooltipPosition,
    children: Children,
) -> impl IntoView {
    let tooltip_class = match tooltip_position {
        TooltipPosition::Top => "tooltip-top",
        TooltipPosition::Bottom => "tooltip-bottom",
    };
    view! {
        <div class="tooltip-wrapper relative inline-block z-50">
            <button
                type="button"
                class=format!("btn btn-ghost btn-xs min-h-0 h-7 px-2 tooltip {tooltip_class}")
                data-tip=label
                aria-label=label
                on:click=move |_| on_click.run(())
            >
                {children()}
            </button>
        </div>
    }
}
