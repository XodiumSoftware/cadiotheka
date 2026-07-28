//! Shared toolbar button component used by the markdown editor and IFC viewer.

use leptos::prelude::*;

/// A small icon-style button with a top tooltip.
#[component]
pub fn ToolbarButton(
    label: &'static str,
    #[prop(into)] on_click: Callback<()>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            type="button"
            class="btn btn-ghost btn-xs min-h-0 h-7 px-2 tooltip tooltip-bottom"
            data-tip=label
            aria-label=label
            on:click=move |_| on_click.run(())
        >
            {children()}
        </button>
    }
}
