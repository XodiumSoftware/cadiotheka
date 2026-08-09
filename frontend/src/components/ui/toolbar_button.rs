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
    /// Tooltip appears to the left of the button.
    Left,
}

/// A small icon-style button with a configurable tooltip and a primary border on hover.
#[component]
pub fn ToolbarButton(
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    #[prop(default = TooltipPosition::Top)] tooltip_position: TooltipPosition,
    #[prop(into, optional)] disabled_overlay: Option<Signal<bool>>,
    #[prop(default = false)] spin_on_click: bool,
    #[prop(into, optional)] on_context_menu: Option<Callback<()>>,
    #[prop(into, optional)] active: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    let tooltip_class = match tooltip_position {
        TooltipPosition::Top => "tooltip-top",
        TooltipPosition::Bottom => "tooltip-bottom",
        TooltipPosition::Left => "tooltip-left",
    };
    let show_stripe = disabled_overlay.unwrap_or_else(|| Signal::derive(|| false));
    let active = active.unwrap_or_else(|| Signal::derive(|| false));
    let spinning = RwSignal::new(false);

    let on_click_wrapper = move |_| {
        if spin_on_click {
            spinning.set(true);
        }
        on_click.run(());
    };

    let on_context_menu_wrapper = move |ev: leptos::web_sys::MouseEvent| {
        ev.prevent_default();
        if let Some(handler) = &on_context_menu {
            handler.run(());
        }
    };

    let aria_label = label.replace('\n', "; ");

    view! {
        <div class="tooltip-wrapper relative inline-block z-50">
            <button
                type="button"
                class=move || {
                    if active.get() {
                        format!("btn btn-xs min-h-0 h-7 px-1.5 tooltip transition-colors border bg-primary border-primary hover:bg-primary {tooltip_class}")
                    } else {
                        format!("btn btn-ghost btn-xs min-h-0 h-7 px-1.5 tooltip transition-colors border border-transparent hover:border-primary {tooltip_class}")
                    }
                }
                data-tip=label
                aria-label=aria_label
                on:click=on_click_wrapper
                on:contextmenu=on_context_menu_wrapper
            >
                <span
                    class=move || {
                        let base = "relative inline-flex items-center justify-center";
                        let spin = if spinning.get() { " animate-spin-once" } else { "" };
                        let color = if active.get() { " text-black" } else { "" };
                        format!("{base}{spin}{color}")
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
