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

/// A small icon-style button with a configurable tooltip and a primary border on hover.
#[component]
pub fn ToolbarButton(
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    #[prop(default = TooltipPosition::Top)] tooltip_position: TooltipPosition,
    #[prop(into, optional)] disabled_overlay: Option<Signal<bool>>,
    #[prop(default = false)] spin_on_click: bool,
    children: Children,
) -> impl IntoView {
    let tooltip_class = match tooltip_position {
        TooltipPosition::Top => "tooltip-top",
        TooltipPosition::Bottom => "tooltip-bottom",
    };
    let show_stripe = disabled_overlay.unwrap_or_else(|| Signal::derive(|| false));
    let spinning = RwSignal::new(false);

    let on_click_wrapper = move |_| {
        if spin_on_click {
            spinning.set(true);
        }
        on_click.run(());
    };

    view! {
        <div class="tooltip-wrapper relative inline-block z-50">
            <button
                type="button"
                class=format!("btn btn-ghost btn-xs min-h-0 h-7 px-1.5 tooltip transition-colors border border-transparent hover:border-primary {tooltip_class}")
                data-tip=label
                aria-label=label
                on:click=on_click_wrapper
            >
                <span
                    class=move || {
                        if spinning.get() {
                            "relative inline-flex items-center justify-center -animate-spin-once".to_string()
                        } else {
                            "relative inline-flex items-center justify-center".to_string()
                        }
                    }
                    on:animationend=move |_| spinning.set(false)
                >
                    {children()}
                    {move || {
                        if show_stripe.get() {
                            view! {
                                <svg
                                    class="absolute inset-0 w-full h-full text-error pointer-events-none"
                                    viewBox="0 0 16 16"
                                    fill="none"
                                    aria-hidden="true"
                                >
                                    <path
                                        d="M2 14 L14 2"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                    />
                                </svg>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                </span>
            </button>
        </div>
    }
}
