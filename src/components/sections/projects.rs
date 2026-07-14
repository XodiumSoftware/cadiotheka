use crate::components::cards::projectcard::ProjectCard;
use crate::components::effects::sectionfade::FadeOverlay;
use crate::context::LayoutContext;
use crate::data::CardData;
use crate::data::load_cards;
use crate::i18n::{t, use_i18n};
use leptos::prelude::*;

#[component]
pub fn ProjectsSection(#[prop(optional)] class: &'static str) -> impl IntoView {
    let i18n = use_i18n();
    let layout = LayoutContext::use_context();
    let cards = load_cards();

    view! {
        <section id="projects" class={format!("relative py-24 sm:py-32 px-6 flex-1 flex flex-col {}", class)}>
            <FadeOverlay />
            <div
                class=move || {
                    if layout.wide.get() {
                        "relative z-10 flex-1 flex flex-col w-full max-w-full"
                    } else {
                        "relative z-10 flex-1 flex flex-col w-full mx-auto max-w-7xl"
                    }
                }
            >
                {if cards.is_empty() {
                    view! {
                        <div class="flex items-center justify-center text-center h-full">
                            <span class="text-base-content/70">{t!(i18n, projects.empty)}</span>
                        </div>
                    }
                        .into_any()
                } else {
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
                            {cards
                                .into_iter()
                                .map(|project: CardData| {
                                    view! {
                                        <ProjectCard props=project.into() />
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                        .into_any()
                }}
            </div>
        </section>
    }
}
