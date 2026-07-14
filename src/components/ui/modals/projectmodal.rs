use crate::components::cards::projectcard::{
    ProjectCardProperties, placeholder_color, placeholder_letter,
};
use crate::components::ui::markdown::MarkdownView;
use crate::components::ui::modals::searchmodal::SearchModal;
use crate::components::ui::overflowrow::{OverflowItem, OverflowRow};
use crate::context::ProjectModalContext;
use crate::data::IconUrl;
use crate::i18n::{t_string, use_i18n};
use leptos::prelude::*;

/// Modal dialog that displays detailed information about a selected project.
#[component]
pub fn ProjectModal() -> impl IntoView {
    let i18n = use_i18n();
    let modal = ProjectModalContext::use_context();
    let on_close = move |_| modal.close();

    view! {
        <SearchModal
            open=modal.open
            on_close=on_close
        >
            {move || {
                let maybe_card = modal.card.get();
                match maybe_card {
                    Some(card) => view! {
                        <ProjectModalContent card=card on_close=on_close />
                    }
                        .into_any(),
                    None => view! {
                        <p class="text-base-content/50 text-sm">{t_string!(i18n, project_modal.empty)}</p>
                    }
                        .into_any(),
                }
            }}
        </SearchModal>
    }
}

#[component]
fn ProjectModalContent(
    #[prop(into)] card: ProjectCardProperties,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let _ = on_close;
    let i18n = use_i18n();
    let letter = placeholder_letter(&card.title);
    let bg = placeholder_color(&card.title);
    let icon_url = card.icon_url.as_ref().map(|IconUrl(url)| url.clone());
    let icon_alt = format!("{} icon", card.title);
    let title = card.title.clone();
    let author = card.author.clone();
    let extended_desc = card.extended_desc.clone();
    let tags = card.tags.clone();
    let platforms = card.supported_platforms.clone();

    view! {
        <div class="space-y-0 flex flex-col min-h-0">
            <div class="flex items-start gap-4">
                {move || {
                    let icon_alt = icon_alt.clone();
                    match icon_url.clone() {
                    Some(url) => view! {
                        <img
                            src={url}
                            alt={icon_alt}
                            class="flex-shrink-0 w-16 h-16 rounded object-cover"
                        />
                    }
                        .into_any(),
                    None => view! {
                        <div
                            class={format!("flex-shrink-0 w-16 h-16 rounded flex items-center justify-center text-white font-bold text-xl {}", bg)}
                            aria-hidden="true"
                        >
                            {letter.clone()}
                        </div>
                    }
                        .into_any(),
                }}}
                <div class="min-w-0 flex-1 flex flex-col gap-1">
                    <h2 class="text-xl font-bold text-primary leading-tight truncate" title={title.clone()}>
                        {title.clone()}
                    </h2>
                    <p class="text-base-content/70 text-sm">
                        {t_string!(i18n, project_modal.by)}
                        <span class="font-semibold text-base-content ml-1" title={author.clone()}>{author.clone()}</span>
                    </p>
                </div>
                <div class="flex items-center gap-1.5 text-xs text-base-content/50 flex-shrink-0">
                    <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">{t_string!(i18n, search.keyboard_esc)}</kbd>
                    <span>{t_string!(i18n, project_modal.hint_dismiss)}</span>
                </div>
            </div>

            <hr class="border-base-content/10" />

            <div class="overflow-y-auto flex-1 min-h-0 py-2 space-y-4">
                {(!tags.is_empty() || !platforms.is_empty()).then(|| view! {
                    <div class="flex flex-wrap items-center gap-2">
                        {(!tags.is_empty()).then(|| view! {
                            <OverflowRow
                                items={tags
                                    .iter()
                                    .map(|tag| OverflowItem::new(tag.label(), tag.color()))
                                    .collect::<Vec<_>>()}
                                max_visible=usize::MAX
                                badge_class="badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap"
                            />
                        }
                            .into_any())}
                        {(!tags.is_empty() && !platforms.is_empty()).then(|| view! {
                            <span class="w-px h-5 bg-base-content/20 self-center" aria-hidden="true" />
                        }
                            .into_any())}
                        {(!platforms.is_empty()).then(|| view! {
                            <OverflowRow
                                items={platforms
                                    .iter()
                                    .map(|platform| OverflowItem::new(platform.label(), platform.color()))
                                    .collect::<Vec<_>>()}
                                max_visible=usize::MAX
                                badge_class="badge badge-sm badge-outline rounded-none border-base-content/10 whitespace-nowrap"
                            />
                        }
                            .into_any())}
                    </div>
                }
                    .into_any())}

                <div>
                    <h3 class="text-sm font-semibold text-base-content mb-1">{t_string!(i18n, project_modal.description)}</h3>
                    <MarkdownView source=extended_desc />
                </div>
            </div>

            <hr class="border-base-content/10" />
        </div>
    }
}
