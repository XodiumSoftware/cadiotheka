use crate::components::cards::project::{ProjectCard, ProjectCardProperties};
use crate::components::ui::corner_frame::CornerFrame;
use crate::components::ui::toggle::ToggleSliderWithSlashLabel;
use crate::contexts::{
    AccountsContext, LayoutContext, ProfileModalContext, ProjectModalContext, ProjectsContext,
    SearchContext,
};
use crate::data::ProjectData;
use crate::engines::SearchEngine;
use crate::ui::effects::section_fade::FadeOverlay;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

/// Computes the number of grid columns for the project card grid based on the
/// current viewport width and the user's wide/narrow layout preference.
#[allow(dead_code)]
fn grid_columns(wide: bool, viewport_width: f64) -> usize {
    if wide {
        if viewport_width >= 1280.0 {
            5
        } else if viewport_width >= 1024.0 {
            4
        } else if viewport_width >= 768.0 {
            3
        } else {
            1
        }
    } else {
        if viewport_width >= 1024.0 {
            3
        } else if viewport_width >= 768.0 {
            2
        } else {
            1
        }
    }
}

#[component]
pub fn ProjectsSection(#[prop(optional)] class: &'static str) -> impl IntoView {
    let layout = LayoutContext::use_context();
    let search = SearchContext::use_context();
    let projects_ctx = ProjectsContext::use_context();

    let (focused_index, set_focused_index) = signal::<Option<usize>>(Some(0));

    let filtered = Memo::new(move |_| {
        let query = search.query.get();
        let projects = projects_ctx.projects.get();
        let parsed = SearchEngine::parse_query(&query);
        SearchEngine::new(projects).search_owned(&parsed)
    });

    Effect::new(move |_| {
        let _ = filtered.get();
        let has_clear_button = !search.query.get().is_empty();
        set_focused_index.update(|idx| {
            let next = idx.and_then(|i| {
                let max = if has_clear_button { 1 } else { 0 };
                if i < max { Some(max) } else { Some(i) }
            });
            *idx = next;
        });
    });

    let grid_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    let grid_columns = move || -> Option<usize> {
        let Some(container) = grid_ref.get() else {
            return Some(1);
        };
        let window = leptos::web_sys::window()?;
        let style = window.get_computed_style(&container).ok().flatten()?;
        let template = style.get_property_value("grid-template-columns").ok()?;
        let trimmed = template.trim();
        if trimmed.is_empty() {
            return Some(1);
        }
        let count = trimmed.split_whitespace().count();
        Some(count.max(1))
    };

    let handle_grid_keydown = Callback::new(move |ev: leptos::web_sys::KeyboardEvent| {
        let Some(current) = focused_index.get() else {
            return;
        };

        let cols = grid_columns().unwrap_or(1);
        if cols == 0 {
            return;
        }

        let card_count = filtered.get().len();
        let has_clear_button = !search.query.get().is_empty();
        let item_count = card_count + if has_clear_button { 1 } else { 0 };
        if item_count == 0 {
            return;
        }

        let next = match ev.key().as_str() {
            "ArrowRight" => Some((current + 1).min(item_count - 1)),
            "ArrowLeft" => Some(current.saturating_sub(1)),
            "ArrowDown" => {
                let target = current + cols;
                if target < item_count {
                    Some(target)
                } else {
                    Some(item_count - 1)
                }
            }
            "ArrowUp" => current.checked_sub(cols),
            "Home" => Some(0),
            "End" => Some(item_count - 1),
            _ => None,
        };

        if let Some(next) = next
            && next != current
        {
            ev.prevent_default();
            set_focused_index.set(Some(next));
        }
    });

    Effect::new(move |_| {
        let Some(index) = focused_index.get() else {
            return;
        };
        let Some(container) = grid_ref.get() else {
            return;
        };
        let Some(children) = container
            .children()
            .dyn_into::<leptos::web_sys::HtmlCollection>()
            .ok()
        else {
            return;
        };
        let item = children.item(index as u32);
        if let Some(el) = item {
            let _ = el
                .dyn_into::<leptos::web_sys::HtmlElement>()
                .map(|html| html.focus());
        }
    });

    view! {
        <section id="projects" class={format!("relative py-16 sm:py-20 px-6 flex-1 flex flex-col {}", class)}>
            <FadeOverlay />
            <FadeOverlay position="spotlight-top" height="96" />
            <FadeOverlay position="spotlight-bottom" height="96" />
            <div
                class=move || {
                    if layout.wide.get() {
                        "relative z-10 flex-1 flex flex-col w-full max-w-full"
                    } else {
                        "relative z-10 flex-1 flex flex-col w-full mx-auto max-w-7xl"
                    }
                }
            >
                <div class="hidden min-[1920px]:flex justify-center mb-8">
                    <ToggleSliderWithSlashLabel
                        checked=layout.wide
                        on_change=move |value| layout.set_wide.set(value)
                        label_left=Signal::derive(move || "Narrow".to_string())
                        label_right=Signal::derive(move || "Wide".to_string())
                        shortcut_hint=Signal::derive(move || "Alt + W".to_string())
                    />
                </div>
                {move || {
                    if projects_ctx.is_loading.get() {
                        return view! {
                            <div class="flex flex-col items-center justify-center text-center h-full gap-4" role="status" aria-live="polite">
                                <span class="loading loading-spinner loading-lg text-primary" aria-hidden="true"></span>
                                <span class="text-base-content/70">"Loading projects..."</span>
                            </div>
                        }
                        .into_any();
                    }

                    let cards = filtered.get();
                    let query = search.query.get();
                    if cards.is_empty() {
                        view! {
                            <div class="flex flex-col items-center justify-center text-center h-full gap-4">
                                <span class="text-error">"No projects found."</span>
                                {move || {
                                    if query.is_empty() {
                                        None
                                    } else {
                                        Some(view! {
                                            <button
                                                type="button"
                                                class="btn btn-outline btn-outline-ghost btn-hover-warning btn-lift gap-1.5"
                                                on:click=move |_| search.set_query.set(String::new())
                                            >
                                                <span>{"Clear Search"}</span>
                                                    <span class="w-1 hidden sm:inline"></span>
                                                    <kbd class="hidden sm:inline-flex px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{"Alt + C"}</kbd>
                                            </button>
                                        })
                                    }
                                }}
                            </div>
                        }
                            .into_any()
                    } else {
                        let query_active = !query.is_empty();
                        let has_clear_button = query_active;
                        let card_offset = if has_clear_button { 1 } else { 0 };
                        view! {
                            <div
                                node_ref=grid_ref
                                class=move || {
                                    if layout.wide.get() {
                                        "grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6 items-stretch"
                                    } else {
                                        "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 items-stretch"
                                    }
                                }
                                on:focusin=move |ev| {
                                    let Some(target) = ev.target() else { return };
                                    let Ok(el) = target.dyn_into::<leptos::web_sys::Element>() else { return };
                                    let Some(parent) = grid_ref.get() else { return };
                                    let Some(children) = parent.children().dyn_into::<leptos::web_sys::HtmlCollection>().ok() else { return };
                                    for i in 0..children.length() {
                                        if children.item(i).as_ref() == Some(&el) {
                                            set_focused_index.set(Some(i as usize));
                                            break;
                                        }
                                    }
                                }
                            >
                                {move || {
                                    if query_active {
                                        Some(view! {
                                            <button
                                                type="button"
                                                class=move || {
                                                    let base = "group btn-lift flex flex-col items-center justify-center h-full w-full bg-white hover:border-primary hover:text-primary border-2 border-base-content/80 p-2 text-left";
                                                    if focused_index.get() == Some(0) {
                                                        format!("{} ring-2 ring-primary ring-offset-2 ring-offset-base-100", base)
                                                    } else {
                                                        base.to_string()
                                                    }
                                                }
                                                tabindex=move || if focused_index.get() == Some(0) { "0" } else { "-1" }
                                                on:keydown=move |ev| handle_grid_keydown.run(ev)
                                                on:click=move |_| search.set_query.set(String::new())
                                            >
                                                <CornerFrame style="square" black=true class="h-full w-full flex flex-col items-center justify-center">
                                                    <span class="font-bold text-lg text-black">{"Click to Clear Search"}</span>
                                                    <span class="text-sm text-black/60 mt-1 hidden sm:block">
                                                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-black bg-black/10 border border-black/30 rounded shadow-kbd">{"Alt + C"}</kbd>
                                                    </span>
                                                </CornerFrame>
                                            </button>
                                        })
                                    } else {
                                        None
                                    }
                                }}
                                {cards
                                    .into_iter()
                                    .enumerate()
                                    .map(|(card_index, project): (usize, ProjectData)| {
                                        let project_for_modal = project.clone();
                                        let project_for_profile = project.clone();
                                        let props: ProjectCardProperties = project.into();
                                        let accounts_ctx = AccountsContext::use_context();
                                        let profile_modal = ProfileModalContext::use_context();
                                        let project_modal = ProjectModalContext::use_context();
                                        let index = card_index + card_offset;
                                        view! {
                                            <ProjectCard
                                                props=props
                                                focused=Signal::derive(move || focused_index.get() == Some(index))
                                                on_click=move |_| project_modal.open(project_for_modal.clone().into())
                                                on_author_click=move |_| {
                                                    let account = accounts_ctx
                                                        .accounts
                                                        .get()
                                                        .into_iter()
                                                        .find(|a| a.id == project_for_profile.author_id);
                                                    if let Some(account) = account {
                                                        profile_modal.open(account);
                                                    }
                                                }
                                                on_key_down=move |ev| handle_grid_keydown.run(ev)
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        }
                            .into_any()
                    }
                }}
            </div>
        </section>
    }
}
