use crate::components::cards::projectcard::ProjectCard;
use crate::components::effects::sectionfade::FadeOverlay;
use crate::components::ui::cornerframe::CornerFrame;
use crate::components::ui::toggle::ToggleSliderWithSlashLabel;
use crate::context::{LayoutContext, SearchContext};
use crate::data::{CardData, load_cards};
use crate::engines::SearchEngine;
use crate::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

#[component]
pub fn ProjectsSection(#[prop(optional)] class: &'static str) -> impl IntoView {
    let i18n = use_i18n();
    let layout = LayoutContext::use_context();
    let search = SearchContext::use_context();
    let all_cards = load_cards();
    let engine = StoredValue::new(SearchEngine::new(all_cards));

    let filtered = Memo::new(move |_| {
        let query = search.query.get();
        let parsed = SearchEngine::parse_query(&query);
        engine.with_value(|engine| engine.search_owned(&parsed))
    });

    view! {
        <section id="projects" class={format!("relative py-16 sm:py-20 px-6 flex-1 flex flex-col {}", class)}>
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
                <div class="hidden min-[1920px]:flex justify-center mb-8">
                    <ToggleSliderWithSlashLabel
                        checked=layout.wide
                        on_change=move |value| layout.set_wide.set(value)
                        label_left=t_string!(i18n, projects.narrow_mode)
                        label_right=t_string!(i18n, projects.wide_mode)
                        shortcut_hint=t_string!(i18n, projects.shortcut_wide)
                    />
                </div>
                {move || {
                    let cards = filtered.get();
                    let query = search.query.get();
                    if cards.is_empty() {
                        view! {
                            <div class="flex flex-col items-center justify-center text-center h-full gap-4">
                                <span class="text-error">{t!(i18n, projects.empty)}</span>
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
                                                <span>{t_string!(i18n, search.clear_button)}</span>
                                                    <span class="w-1"></span>
                                                    <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{t_string!(i18n, search.shortcut_clear)}</kbd>
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
                                                class="group flex flex-col items-center justify-center h-full w-full bg-white hover:border-primary hover:text-primary border-2 border-base-content/80 p-2 text-left"
                                                on:click=move |_| search.set_query.set(String::new())
                                            >
                                                <CornerFrame style="square" class="h-full w-full flex flex-col items-center justify-center">
                                                    <span class="font-bold text-lg text-black">{t_string!(i18n, search.clear_card_title)}</span>
                                                    <span class="text-sm text-black/60 mt-1">
                                                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-black bg-black/10 border border-black/30 rounded shadow-kbd">{t_string!(i18n, search.shortcut_clear)}</kbd>
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
                                    .map(|project: CardData| {
                                        view! {
                                            <ProjectCard props=project.into() />
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
