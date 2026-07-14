use crate::components::cards::projectcard::ProjectCard;
use crate::components::effects::hexgrid::HexPattern;
use crate::components::effects::sectionfade::FadeOverlay;
use crate::components::ui::cornerframe::CornerFrame;
use crate::components::ui::datagrid::data_grid;
use crate::github::{Repo, fetch_repos};
use crate::i18n::{t, t_string, use_i18n};
use leptos::prelude::*;

#[component]
pub fn ProjectsSection() -> impl IntoView {
    let i18n = use_i18n();
    let (retry_count, set_retry_count) = signal(0u32);
    let resource = LocalResource::new(move || {
        let _ = retry_count.get();
        async move { fetch_repos().await }
    });

    let retry = move || {
        set_retry_count.update(|n| *n += 1);
    };

    view! {
        <section id="projects" class="relative py-24 sm:py-32 px-6">
            <HexPattern />
            <FadeOverlay />
            <div class="mx-auto max-w-7xl relative z-10">
                <div class="flex gap-8 items-stretch">
                    <div class="flex-shrink-0 flex items-center bg-surface-light p-2 relative">
                        <CornerFrame
                            style="square"
                            black=true
                            class="h-full w-full flex items-center justify-center"
                        >
                            <h2 class="text-3xl font-bold tracking-tight text-transparent bg-base-100 bg-clip-text sm:text-4xl [writing-mode:vertical-rl] rotate-180 whitespace-nowrap">
                                {t!(i18n, projects.title)}
                            </h2>
                        </CornerFrame>
                    </div>
                    <div class="flex-1 min-w-0">
                        {data_grid(
                            resource,
                            move || t!(i18n, projects.empty),
                            |projects: Vec<Repo>| {
                                view! {
                                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 items-stretch">
                                        {projects
                                            .into_iter()
                                            .map(|project| {
                                                view! {
                                                    <ProjectCard props=project.into() />
                                                }
                                            })
                                            .collect_view()}
                                    </div>
                                }
                            },
                            Some(retry),
                            t_string!(i18n, projects.retry),
                        )}
                    </div>
                </div>
            </div>
        </section>
    }
}
