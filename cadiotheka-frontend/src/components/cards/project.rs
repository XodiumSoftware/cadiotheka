use crate::components::ui::corner_frame::CornerFrame;
use crate::components::ui::overflow_row::{OverflowItem, OverflowRow};
use crate::contexts::{CurrentUserContext, ProjectModalContext, ProjectsContext};
use crate::data::{IconUrl, ProjectData};
use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use crate::utils::{
    format_number, format_number_full, format_time_ago, format_time_full, placeholder_color,
    placeholder_letter,
};
use leptos::prelude::*;

#[derive(Clone)]
pub struct ProjectCardProperties {
    pub id: String,
    pub title: String,
    pub author: String,
    pub author_id: String,
    pub author_username: String,
    pub collaborator_ids: Vec<String>,
    pub description: String,
    pub extended_desc: String,
    pub tags: Vec<Tag>,
    pub supported_platforms: Vec<Platform>,
    pub downloads: u64,
    pub favorites: Vec<String>,
    pub timestamp: time::OffsetDateTime,
    pub icon_url: Option<IconUrl>,
}

impl From<ProjectData> for ProjectCardProperties {
    fn from(project: ProjectData) -> Self {
        project_card_properties_from_project_data(project)
    }
}

pub fn project_card_properties_from_project_data(project: ProjectData) -> ProjectCardProperties {
    let description = if project.description.trim().is_empty() {
        "(No description)".to_string()
    } else {
        project.description
    };
    let extended_desc = if project.extended_desc.trim().is_empty() {
        description.clone()
    } else {
        project.extended_desc
    };
    ProjectCardProperties {
        id: project.id,
        title: project.title,
        author: project.author,
        author_id: project.author_id,
        author_username: project.author_username,
        collaborator_ids: project.collaborator_ids,
        description,
        extended_desc,
        tags: project.tags,
        supported_platforms: project.supported_platforms,
        downloads: project.downloads,
        favorites: project.favorites,
        timestamp: project.timestamp,
        icon_url: project.icon_url,
    }
}

pub use project_card_properties_from_project_data as from_project_data;

#[component]
pub fn DownloadIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
    }
}

#[component]
pub fn HeartIcon(#[prop(optional, into)] filled: Signal<bool>) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill=move || if filled.get() { "currentColor" } else { "none" }
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />
        </svg>
    }
}

#[component]
pub fn ClockIcon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <circle cx="12" cy="12" r="10" />
            <polyline points="12 6 12 12 16 14" />
        </svg>
    }
}

#[component]
pub fn ProjectCard(
    #[prop(into)] props: ProjectCardProperties,
    #[prop(into)] on_click: Callback<()>,
    #[prop(into)] on_author_click: Callback<()>,
) -> impl IntoView {
    let letter = placeholder_letter(&props.title);
    let bg = placeholder_color(&props.title);
    let icon_url = props.icon_url.as_ref().map(|IconUrl(url)| url.clone());
    let current_user = CurrentUserContext::use_context();
    let projects_ctx = ProjectsContext::use_context();
    let project_modal = ProjectModalContext::use_context();

    let downloads = props.downloads;
    let timestamp = props.timestamp;
    let project_id = props.id.clone();
    let project_id_for_button = props.id.clone();
    let favorite_count = Signal::derive({
        let project_id = project_id.clone();
        move || {
            projects_ctx
                .projects
                .get()
                .into_iter()
                .find(|project| project.id == project_id)
                .map(|project| project.favorites.len())
                .unwrap_or(0)
        }
    });
    let is_favorited = Signal::derive({
        let project_id = project_id.clone();
        move || {
            let Some(account) = current_user.account.get() else {
                return false;
            };
            projects_ctx
                .projects
                .get()
                .into_iter()
                .find(|project| project.id == project_id)
                .is_some_and(|project| {
                    project
                        .favorites
                        .iter()
                        .any(|user_id| user_id == &account.id)
                })
        }
    });

    let card_title = props.title.clone();
    let card_author = props.author.clone();
    let card_author_username = props.author_username.clone();
    let card_title_for_favorite_label = props.title.clone();
    let favorite_aria_label = Signal::derive(move || {
        if is_favorited.get() {
            format!("Remove {} from favorites", card_title_for_favorite_label)
        } else {
            format!("Add {} to favorites", card_title_for_favorite_label)
        }
    });
    let card_title_for_icon_alt = card_title.clone();
    let icon_alt = Signal::derive(move || format!("{} icon", card_title_for_icon_alt.clone()));
    let card_title_for_aria = card_title.clone();
    let card_author_for_aria = card_author.clone();
    let aria_label = Signal::derive(move || {
        format!(
            "Open details for {} by {}",
            card_title_for_aria.clone(),
            card_author_for_aria.clone()
        )
    });
    let tags = props.tags.clone();
    let platforms = props.supported_platforms.clone();

    let description = props.description.clone();

    view! {
        <article
            class="btn-lift hover:border-primary block h-full p-2 cursor-pointer"
            on:click=move |_| on_click.run(())
            role="button"
            tabindex="0"
            aria-label=aria_label
        >
            <CornerFrame style="square" class="h-full">
                <div class="card bg-ghost h-full rounded-none">
                    <div class="card-body p-4">
                        <div class="flex items-start gap-3">
                            {move || {
                                match icon_url.clone() {
                                    Some(url) => view! {
                                        <img
                                            src={url}
                                            alt=icon_alt
                                            class="flex-shrink-0 w-10 h-10 object-cover"
                                            loading="lazy"
                                        />
                                    }
                                        .into_any(),
                                    None => view! {
                                        <div class=format!("flex-shrink-0 w-10 h-10 flex items-center justify-center text-white font-bold {}", bg)
                                            aria-hidden="true"
                                        >
                                            {letter.clone()}
                                        </div>
                                    }
                                        .into_any(),
                                }
                            }}
                            <div class="min-w-0 flex-1 flex flex-col gap-2">
                                <div class="flex items-start gap-2">
                                    <h2 class="card-title text-primary text-base leading-tight min-w-0 flex-1">
                                        <span class="truncate" title={card_title.clone()}>{card_title.clone()}</span>
                                        <span class="text-base-content/60 font-normal">{" by "}</span>
                                        <button
                                            type="button"
                                            class="text-base-content font-semibold truncate hover:text-primary hover:underline"
                                            title={format!("@{}", card_author_username)}
                                            on:click=move |ev| {
                                                ev.stop_propagation();
                                                on_author_click.run(());
                                            }
                                        >
                                            {card_author.clone()}
                                        </button>
                                    </h2>
                                    <Show when=move || current_user.account.get().is_some()>
                                        <button
                                            type="button"
                                            class=move || {
                                                if is_favorited.get() {
                                                    "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-error"
                                                } else {
                                                    "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-error"
                                                }
                                            }
                                            aria-label=favorite_aria_label
                                            title=move || {
                                                if is_favorited.get() {
                                                    "Remove favorite".to_string()
                                                } else {
                                                    "Add favorite".to_string()
                                                }
                                            }
                                            on:click={
                                                let project_id = project_id_for_button.clone();
                                                move |ev| {
                                                    ev.stop_propagation();
                                                    let project_id = project_id.clone();
                                                    let set_projects = projects_ctx.set_projects;
                                                    let modal_set_card = project_modal.set_card;
                                                    leptos::task::spawn_local(async move {
                                                        if let Some(updated) = ProjectsContext::toggle_favorite(&project_id).await {
                                                            let updated_for_modal = updated.clone();
                                                            set_projects.update(|projects| {
                                                                if let Some(project) = projects.iter_mut().find(|project| project.id == updated.id) {
                                                                    *project = updated.clone();
                                                                }
                                                            });
                                                            modal_set_card.update(|card| {
                                                                if let Some(card) = card.as_mut()
                                                                    && card.id == updated_for_modal.id
                                                                {
                                                                    card.favorites = updated_for_modal.favorites.clone();
                                                                }
                                                            });
                                                        }
                                                    });
                                                }
                                            }
                                        >
                                            <HeartIcon filled=Signal::derive(move || is_favorited.get()) />
                                        </button>
                                    </Show>
                                </div>

                                <div class="flex flex-nowrap items-center gap-1 overflow-hidden">
                                    <OverflowRow
                                        items={tags
                                            .iter()
                                            .map(|tag| OverflowItem::new(tag.label(), tag.color()))
                                            .collect::<Vec<_>>()}
                                        max_visible=2
                                        badge_class="badge badge-xs badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap"
                                    />
                                    {(!tags.is_empty() && !platforms.is_empty()).then(|| {
                                        view! {
                                            <span class="w-px h-4 bg-base-content/20 self-center mx-1 flex-shrink-0" aria-hidden="true" />
                                        }
                                            .into_any()
                                    })}
                                    <OverflowRow
                                        items={platforms
                                            .iter()
                                            .map(|platform| OverflowItem::new(platform.label(), platform.color()))
                                            .collect::<Vec<_>>()}
                                        max_visible=1
                                        badge_class="badge badge-xs badge-outline rounded-none border-base-content/10 whitespace-nowrap"
                                    />
                                </div>
                            </div>
                        </div>

                        <hr class="border-base-content/10 my-3" />

                        <p class="text-base-content/70 flex-grow text-sm">{description}</p>

                        <hr class="border-base-content/10 my-3" />

                        <div class="flex items-center gap-4 text-base-content/60 text-sm">
                            <span
                                class="flex items-center gap-1"
                                title={move || format!("{} downloads", format_number_full(downloads))}
                            >
                                <DownloadIcon />
                                {move || format_number(downloads)}
                            </span>
                            <span
                                class="flex items-center gap-1"
                                title={move || format!("{} favorites", format_number_full(favorite_count.get() as u64))}
                            >
                                <HeartIcon filled=Signal::derive(move || is_favorited.get()) />
                                {move || format_number(favorite_count.get() as u64)}
                            </span>
                            <span
                                class="flex items-center gap-1"
                                title={move || format!("Updated {}", format_time_full(timestamp))}
                            >
                                <ClockIcon />
                                {move || format_time_ago(timestamp)}
                            </span>
                        </div>
                    </div>
                </div>
            </CornerFrame>
        </article>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::IconUrl;

    #[test]
    fn test_project_card_properties_from_project_data() {
        let project = ProjectData {
            id: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".to_owned(),
            title: "Gear".to_owned(),
            author: "Author".to_owned(),
            author_id: "b2c3d4e5-f6a7-8901-bcde-f12345678901".to_owned(),
            author_username: "author".to_owned(),
            collaborator_ids: vec![],
            description: "A gear.".to_owned(),
            extended_desc: "A gear with an **extended** markdown description.".to_owned(),
            tags: vec![Tag::Model3d],
            supported_platforms: vec![Platform::Blender],
            downloads: 1234,
            favorites: vec!["user-1".to_owned(), "user-2".to_owned()],
            timestamp: time::macros::datetime!(2024-01-01 00:00:00 UTC),
            icon_url: Some(IconUrl("https://example.com/gear.svg".to_owned())),
        };
        let props: ProjectCardProperties = project.into();
        assert_eq!(props.title, "Gear");
        assert_eq!(props.author, "Author");
        assert_eq!(props.tags.len(), 1);
        assert_eq!(props.supported_platforms.len(), 1);
    }
}
