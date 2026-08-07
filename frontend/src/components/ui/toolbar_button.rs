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
///
/// When `active` is `true` the button shows a primary-colored border instead
/// of swapping icons.
#[component]
pub fn ToolbarButton(
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    #[prop(default = TooltipPosition::Top)] tooltip_position: TooltipPosition,
    #[prop(into, optional)] active: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    let tooltip_class = match tooltip_position {
        TooltipPosition::Top => "tooltip-top",
        TooltipPosition::Bottom => "tooltip-bottom",
    };
    let active = active.unwrap_or_else(|| Signal::derive(|| false));

    view! {
        <div class="tooltip-wrapper relative inline-block z-50">
            <button
                type="button"
                class=move || {
                    let base = "btn btn-ghost btn-xs min-h-0 h-7 px-1.5 tooltip transition-colors border border-transparent";
                    let active_class = if active.get() { " border-primary" } else { "" };
                    format!("{base} {tooltip_class}{active_class}")
                }
                data-tip=label
                aria-label=label
                aria-pressed=move || active.get().to_string()
                on:click=move |_| on_click.run(())
            >
                {children()}
            </button>
        </div>
    }
}
