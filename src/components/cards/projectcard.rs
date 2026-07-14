use crate::components::ui::cornerframe::CornerFrame;
use crate::components::ui::overflowrow::{OverflowItem, OverflowRow};
use crate::context::ProjectModalContext;
use crate::data::{CardData, IconUrl};
use crate::i18n::use_i18n;
use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use crate::utils::{format_number, format_number_full, format_time_ago, format_time_full};
use leptos::prelude::*;

#[derive(Clone)]
pub struct ProjectCardProperties {
    pub title: String,
    pub author: String,
    pub description: String,
    pub extended_desc: String,
    pub tags: Vec<Tag>,
    pub supported_platforms: Vec<Platform>,
    pub downloads: u64,
    pub favorites: u64,
    pub timestamp: time::OffsetDateTime,
    pub icon_url: Option<IconUrl>,
}

impl From<CardData> for ProjectCardProperties {
    fn from(card: CardData) -> Self {
        project_card_properties_from_card_data(card)
    }
}

pub fn project_card_properties_from_card_data(card: CardData) -> ProjectCardProperties {
    let description = if card.description.trim().is_empty() {
        "(No description)".to_string()
    } else {
        card.description
    };
    let extended_desc = if card.extended_desc.trim().is_empty() {
        description.clone()
    } else {
        card.extended_desc
    };
    ProjectCardProperties {
        title: card.title,
        author: card.author,
        description,
        extended_desc,
        tags: card.tags,
        supported_platforms: card.supported_platforms,
        downloads: card.downloads,
        favorites: card.favorites,
        timestamp: card.timestamp,
        icon_url: card.icon_url,
    }
}

pub use project_card_properties_from_card_data as from_card_data;

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
pub fn HeartIcon() -> impl IntoView {
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

pub fn placeholder_letter(title: &str) -> String {
    title
        .chars()
        .next()
        .unwrap_or('?')
        .to_uppercase()
        .to_string()
}

pub fn placeholder_color(title: &str) -> &'static str {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let palette: [&'static str; 8] = [
        "bg-red-500",
        "bg-orange-500",
        "bg-yellow-500",
        "bg-green-500",
        "bg-cyan-500",
        "bg-blue-500",
        "bg-purple-500",
        "bg-pink-500",
    ];

    let mut hasher = DefaultHasher::new();
    title.hash(&mut hasher);
    let hash = hasher.finish();
    palette[(hash as usize) % palette.len()]
}

#[component]
pub fn ProjectCard(props: ProjectCardProperties) -> impl IntoView {
    let _i18n = use_i18n();
    let letter = placeholder_letter(&props.title);
    let bg = placeholder_color(&props.title);
    let icon_url = props.icon_url.as_ref().map(|IconUrl(url)| url.clone());
    let title = props.title.clone();
    let downloads = props.downloads;
    let favorites = props.favorites;
    let timestamp = props.timestamp;

    let props_for_modal = props.clone();
    let open_modal = move |_| {
        ProjectModalContext::use_context().open(props_for_modal.clone());
    };
    let aria_label = format!("Open details for {} by {}", props.title, props.author);
    let card_title = props.title.clone();
    let card_author = props.author.clone();
    let tags = props.tags.clone();
    let platforms = props.supported_platforms.clone();

    let description = props.description.clone();

    view! {
        <article
            class="btn-lift hover:border-primary block h-full p-2 cursor-pointer"
            on:click=open_modal
            role="button"
            tabindex="0"
            aria-label=aria_label
        >
            <CornerFrame style="square" class="h-full">
                <div class="card bg-ghost h-full rounded-none">
                    <div class="card-body p-4">
                        <div class="flex items-start gap-3">
                            {move || match icon_url.clone() {
                                Some(url) => view! {
                                    <img
                                        src={url}
                                        alt={format!("{} icon", title.clone())}
                                        class="flex-shrink-0 w-10 h-10 rounded object-cover"
                                        loading="lazy"
                                    />
                                }
                                    .into_any(),
                                None => view! {
                                    <div
                                        class={format!("flex-shrink-0 w-10 h-10 rounded flex items-center justify-center text-white font-bold {}", bg)}
                                        aria-hidden="true"
                                    >
                                        {letter.clone()}
                                    </div>
                                }
                                    .into_any(),
                            }}
                            <div class="min-w-0 flex-1 flex flex-col gap-2">
                                <h2 class="card-title text-primary text-base leading-tight">
                                    <span class="truncate" title={card_title.clone()}>{card_title.clone()}</span>
                                    <span class="text-base-content/60 font-normal">{" by "}</span>
                                    <span class="text-base-content font-semibold truncate" title={card_author.clone()}>
                                        {card_author.clone()}
                                    </span>
                                </h2>

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
                                title={move || format!("{} favorites", format_number_full(favorites))}
                            >
                                <HeartIcon />
                                {move || format_number(favorites)}
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
    fn test_placeholder_letter() {
        assert_eq!(placeholder_letter("Blender"), "B");
        assert_eq!(placeholder_letter("freecad"), "F");
        assert_eq!(placeholder_letter(""), "?");
    }

    #[test]
    fn test_placeholder_color_is_deterministic() {
        let a = placeholder_color("abc");
        let b = placeholder_color("abc");
        assert_eq!(a, b);
    }

    #[test]
    fn test_project_card_properties_from_card_data() {
        let card = CardData {
            title: "Gear".to_owned(),
            author: "Author".to_owned(),
            description: "A gear.".to_owned(),
            extended_desc: "A gear with an **extended** markdown description.".to_owned(),
            tags: vec![Tag::Model3d],
            supported_platforms: vec![Platform::Blender],
            downloads: 1234,
            favorites: 56,
            timestamp: time::macros::datetime!(2024-01-01 00:00:00 UTC),
            icon_url: Some(IconUrl("https://example.com/gear.svg".to_owned())),
        };
        let props: ProjectCardProperties = card.into();
        assert_eq!(props.title, "Gear");
        assert_eq!(props.author, "Author");
        assert_eq!(props.tags.len(), 1);
        assert_eq!(props.supported_platforms.len(), 1);
    }
}
