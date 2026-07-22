use crate::components::ui::corner_frame::CornerFrame;
use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos::web_sys;

const FOCUSABLE_SELECTORS: &str = "button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex=\"-1\"])";

/// Returns the list of currently focusable elements inside `container`, ordered
/// by their position in the document.
fn focusable_elements(container: &web_sys::Element) -> Vec<web_sys::HtmlElement> {
    container
        .query_selector_all(FOCUSABLE_SELECTORS)
        .ok()
        .map(|nodes: web_sys::NodeList| {
            (0..nodes.length())
                .filter_map(|i| {
                    nodes
                        .item(i)
                        .and_then(|n: web_sys::Node| n.dyn_into::<web_sys::HtmlElement>().ok())
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A reusable modal dialog with a search-modal visual style.
#[component]
pub fn SearchModal(
    #[prop(into)] open: Signal<bool>,
    #[prop(into)] on_close: Callback<()>,
    #[prop(into, default = Callback::new(|_| {}))] on_inner_click: Callback<()>,
    #[prop(into, default = "w-full max-w-lg max-h-[80vh] flex flex-col".to_string())]
    container_class: String,
    children: Children,
) -> impl IntoView {
    let children_view = children();
    let backdrop_ref: NodeRef<leptos::html::Div> = NodeRef::new();
    let panel_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", {
            let on_close = on_close;
            move |ev| {
                if ev.key().eq_ignore_ascii_case("escape") {
                    ev.prevent_default();
                    on_close.run(());
                }
            }
        });
    });

    Effect::new(move |_| {
        if let Some(body) = leptos::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            if open.get() {
                body.class_list().add_1("overflow-hidden").ok();
            } else {
                body.class_list().remove_1("overflow-hidden").ok();
            }
        }
    });

    leptos::prelude::on_cleanup(move || {
        if let Some(body) = leptos::web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.body())
        {
            body.class_list().remove_1("overflow-hidden").ok();
        }
    });

    view! {
        <div
            node_ref=backdrop_ref
            class=move || {
                if open.get() {
                    "fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm"
                } else {
                    "hidden"
                }
            }
            on:click=move |ev: leptos::web_sys::MouseEvent| {
                if let Some(target) = ev.target()
                    && let Ok(clicked) = target.dyn_into::<leptos::web_sys::Node>()
                    && let Some(backdrop) = backdrop_ref.get()
                {
                    if backdrop.is_same_node(Some(&clicked)) {
                        on_close.run(());
                    } else {
                        on_inner_click.run(());
                    }
                }
            }
        >
            <div
                class=container_class
                node_ref=panel_ref
                role="dialog"
                aria-modal="true"
                aria-label="Search"
                on:keydown=move |ev: web_sys::KeyboardEvent| {
                    if ev.key().eq_ignore_ascii_case("tab") {
                        let Some(panel) = panel_ref.get() else { return };
                        let focusable = focusable_elements(&panel);
                        if focusable.is_empty() {
                            return;
                        }
                        let active = leptos::web_sys::window()
                            .and_then(|w| w.document())
                            .and_then(|d| d.active_element())
                            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok());
                        let first = focusable.first();
                        let last = focusable.last();
                        let shift = ev.shift_key();
                        if (!shift && active.as_ref() == last)
                            || (shift && active.as_ref() == first)
                        {
                            ev.prevent_default();
                            let target = if shift { last } else { first };
                            target.map(|el| el.focus().ok());
                        }
                    }
                }
            >
                <div class="block p-2 bg-base-100 border-2 border-primary h-full flex flex-col">
                    <CornerFrame style="square" class="w-full h-full">
                        <div class="h-full rounded-none p-6 flex flex-col">
                            {children_view}
                        </div>
                    </CornerFrame>
                </div>
            </div>
        </div>
    }
}
