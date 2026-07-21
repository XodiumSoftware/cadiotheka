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

#[component]
pub fn ProjectsSection(#[prop(optional)] class: &'static str) -> impl IntoView {
    let layout = LayoutContext::use_context();
    let search = SearchContext::use_context();
    let projects_ctx = ProjectsContext::use_context();

    let filtered = Memo::new(move |_| {
        let query = search.query.get();
        let projects = projects_ctx.projects.get();
        let parsed = SearchEngine::parse_query(&query);
        SearchEngine::new(projects).search_owned(&parsed)
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
                        view! {
                            <div
                                class=move || {
                                    if layout.wide.get() {
                                        "grid grid-cols-1 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6 items-stretch"
                                    } else {
                                        "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 items-stretch"
                                    }
                                }
                            >
                                {move || {
                                    if query_active {
                                        Some(view! {
                                            <button
                                                type="button"
                                                class="group btn-lift flex flex-col items-center justify-center h-full w-full bg-white hover:border-primary hover:text-primary border-2 border-base-content/80 p-2 text-left"
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
                                    .map(|project: ProjectData| {
                                        let project_for_modal = project.clone();
                                        let project_for_profile = project.clone();
                                        let props: ProjectCardProperties = project.into();
                                        let accounts_ctx = AccountsContext::use_context();
                                        let profile_modal = ProfileModalContext::use_context();
                                        let project_modal = ProjectModalContext::use_context();
                                        view! {
                                            <ProjectCard
                                                props=props
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
