use crate::components::cards::project::{HeartIcon, ProjectCardProperties};
use crate::components::ui::markdown::MarkdownView;
use crate::components::ui::markdown_editor::MarkdownEditor;
use crate::components::ui::modals::search::SearchModal;
use crate::components::ui::project_icon_picker::ProjectIconPicker;

use crate::contexts::{
    AccountsContext, CurrentUserContext, ProfileModalContext, ProjectModalContext, ProjectsContext,
    SearchContext,
};
use crate::data::{
    AccountData, AccountRole, delete_project, fetch_projects, update_project_collaborators,
    update_project_description, update_project_extended_desc, update_project_platforms,
    update_project_tags, update_project_title, upload_project_icon,
};
use crate::utils::{placeholder_color, placeholder_letter};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

const MAX_TITLE_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MAX_EXTENDED_DESC_LENGTH: usize = 5000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProjectDetailsTab {
    About,
    Viewer3d,
    Versions,
}

/// Modal dialog that displays detailed information about a selected project.
#[component]
pub fn ProjectModal() -> impl IntoView {
    let modal = ProjectModalContext::use_context();
    let on_close = move |_| modal.close();

    view! {
        <SearchModal
            open=modal.open
            on_close=on_close
            container_class="w-full max-w-6xl h-full max-h-[90vh] flex flex-col"
        >
            {move || {
                let maybe_card = modal.card.get();
                match maybe_card {
                    Some(card) => view! {
                        <ProjectModalContent card=card />
                    }
                        .into_any(),
                    None => view! {
                        <p class="text-base-content/50 text-sm">No project selected.</p>
                    }
                        .into_any(),
                }
            }}
        </SearchModal>
    }
}

fn avatar_button(account: &AccountData, class: Option<String>) -> impl IntoView + use<> {
    let display_name = account.display_name.clone();
    let avatar_alt = format!("{}'s avatar", display_name);
    let avatar_letter = placeholder_letter(&display_name);
    let avatar_bg = placeholder_color(&display_name);
    let size_class = class.unwrap_or_else(|| "w-12 h-12".to_string());
    let url = account.avatar_url.clone();
    view! {
        <div
            class=format!("{} border border-base-content/10 overflow-hidden flex items-center justify-center text-white font-bold text-lg tooltip tooltip-top {}", size_class, avatar_bg)
            data-tip=display_name.clone()
            aria-label=avatar_alt.clone()
        >
            {url.map(|url| {
                view! {
                    <img class="w-full h-full object-cover" src=url alt=avatar_alt.clone() />
                }
                    .into_any()
            }).unwrap_or_else(move || {
                view! {
                    <span>{avatar_letter.clone()}</span>
                }
                    .into_any()
            })}
        </div>
    }
}

fn trash_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
        </svg>
    }
}

fn warning_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d="M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
        </svg>
    }
}

fn edit_pencil_icon(class: &'static str) -> impl IntoView {
    view! {
        <svg
            class=class
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
        </svg>
    }
}

#[component]
fn EditableChipSection<T: Clone + PartialEq + Send + Sync + 'static>(
    #[allow(unused_variables)] title: &'static str,
    #[allow(unused_variables)] aria_label: &'static str,
    #[allow(unused_variables)] items: Vec<T>,
    all_items: Vec<T>,
    editing: Signal<bool>,
    on_cancel: Callback<()>,
    on_toggle: Callback<T>,
    on_save: Callback<Vec<T>>,
    on_item_click: Callback<T>,
    label_fn: fn(&T) -> &'static str,
    color_fn: fn(&T) -> &'static str,
    selected_items: Signal<Vec<T>>,
    badge_class: &'static str,
) -> impl IntoView {
    view! {
        <div class="space-y-3">
            <h3 class="text-sm font-semibold text-base-content">{title}</h3>
            {move || {
                if editing.get() {
                    let current_selected = selected_items.get();
                    view! {
                        <div class="space-y-2">
                            <div class="flex flex-wrap gap-2" role="group" aria-label=aria_label>
                                {all_items.iter().map(|item| {
                                    let item_for_class = item.clone();
                                    let item_for_aria = item.clone();
                                    view! {
                                        <button
                                            type="button"
                                            class=move || {
                                                let selected = selected_items.get().contains(&item_for_class);
                                                format!(
                                                    "badge badge-sm badge-outline rounded-none cursor-pointer transition-colors {}",
                                                    if selected {
                                                        "bg-primary/20 border-primary text-primary"
                                                    } else {
                                                        "border-base-content/20 text-base-content/70 hover:border-primary/50"
                                                    }
                                                )
                                            }
                                            on:click={
                                                let item = item.clone();
                                                move |_| on_toggle.run(item.clone())
                                            }
                                            aria-pressed=move || selected_items.get().contains(&item_for_aria).to_string()
                                        >
                                            {label_fn(item)}
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                            <div class="flex justify-end gap-2">
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs"
                                    on:click=move |_| on_cancel.run(())
                                >"Cancel"</button>
                                <button
                                    type="button"
                                    class="btn btn-primary btn-xs"
                                    on:click=move |_| on_save.run(current_selected.clone())
                                >"Save"</button>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <div class="flex flex-wrap gap-2" role="group" aria_label=aria_label>
                            {items.iter().map(|item| {
                                let item_for_click = item.clone();
                                view! {
                                    <button
                                        type="button"
                                        class=format!("{} {}", badge_class, color_fn(item))
                                        on:click=move |_| on_item_click.run(item_for_click.clone())
                                    >
                                        {label_fn(item)}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    }
                        .into_any()
                }
            }}
        </div>
    }
}

#[component]
fn ProjectModalContent(#[prop(into)] card: ProjectCardProperties) -> impl IntoView {
    let current_user = CurrentUserContext::use_context();
    let projects_ctx = ProjectsContext::use_context();
    let modal = ProjectModalContext::use_context();
    let profile_modal = ProfileModalContext::use_context();
    let search = SearchContext::use_context();
    let is_editable = Signal::derive({
        let author_id = card.author_id.clone();
        move || {
            current_user
                .account
                .get()
                .is_some_and(|me| me.role == AccountRole::Admin || me.id == author_id)
        }
    });

    let (active_tab, set_active_tab) = signal(ProjectDetailsTab::About);

    let (editing_title, set_editing_title) = signal(false);
    let (draft_title, set_draft_title) = signal(card.title.clone());
    let (title, set_title) = signal(card.title.clone());

    let (editing_description, set_editing_description) = signal(false);
    let (draft_description, set_draft_description) = signal(card.description.clone());
    let (description, set_description) = signal(card.description.clone());

    let (editing_tags, set_editing_tags) = signal(false);
    let (draft_tags, set_draft_tags) = signal(card.tags.clone());
    let (tags, set_tags) = signal(card.tags.clone());

    let (editing_platforms, set_editing_platforms) = signal(false);
    let (draft_platforms, set_draft_platforms) = signal(card.supported_platforms.clone());
    let (supported_platforms, set_supported_platforms) = signal(card.supported_platforms.clone());

    let (editing_extended, set_editing_extended) = signal(false);
    let (draft_extended, set_draft_extended) = signal(card.extended_desc.clone());
    let (extended_desc, set_extended_desc) = signal(card.extended_desc.clone());

    let (editing_collaborators, set_editing_collaborators) = signal(false);
    let (collaborator_ids, set_collaborator_ids) = signal(card.collaborator_ids.clone());
    let (draft_collaborator_ids, set_draft_collaborator_ids) =
        signal(card.collaborator_ids.clone());

    let icon_input_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let (icon_url, set_icon_url) = signal(card.icon_url.clone());

    let (edit_mode, set_edit_mode) = signal(false);

    let (show_delete_confirm, set_show_delete_confirm) = signal(false);
    let (delete_confirm_input, set_delete_confirm_input) = signal(String::new());
    let (is_deleting, set_is_deleting) = signal(false);
    let can_delete =
        Signal::derive(move || delete_confirm_input.get().trim() == title.get().trim());

    let project_id = card.id.clone();

    let delete_project_click = {
        let project_id = project_id.clone();
        let set_projects = projects_ctx.set_projects;
        Callback::new(move |_| {
            let project_id = project_id.clone();
            let set_projects = set_projects;
            leptos::task::spawn_local(async move {
                set_is_deleting.set(true);
                let success = delete_project(&project_id).await;
                set_is_deleting.set(false);
                if success {
                    set_delete_confirm_input.set(String::new());
                    set_show_delete_confirm.set(false);
                    let refreshed = fetch_projects().await;
                    set_projects.set(refreshed);
                    modal.close();
                }
            });
        })
    };

    let start_edit_title = move || {
        set_draft_title.set(title.get());
        set_editing_title.set(true);
    };

    let cancel_edit_title = move || {
        set_draft_title.set(title.get());
        set_editing_title.set(false);
    };

    let start_edit_description = move || {
        set_draft_description.set(description.get());
        set_editing_description.set(true);
    };

    let cancel_edit_description = move || {
        set_draft_description.set(description.get());
        set_editing_description.set(false);
    };

    let start_edit_tags = move || {
        set_draft_tags.set(tags.get());
        set_editing_tags.set(true);
    };

    let cancel_edit_tags = move || {
        set_draft_tags.set(tags.get());
        set_editing_tags.set(false);
    };

    let start_edit_platforms = move || {
        set_draft_platforms.set(supported_platforms.get());
        set_editing_platforms.set(true);
    };

    let cancel_edit_platforms = move || {
        set_draft_platforms.set(supported_platforms.get());
        set_editing_platforms.set(false);
    };

    let start_edit_extended = move || {
        set_draft_extended.set(extended_desc.get());
        set_editing_extended.set(true);
    };

    let cancel_edit_extended = move || {
        set_draft_extended.set(extended_desc.get());
        set_editing_extended.set(false);
    };

    let start_edit_collaborators = move || {
        set_draft_collaborator_ids.set(collaborator_ids.get());
        set_editing_collaborators.set(true);
    };

    let cancel_edit_collaborators = move || {
        set_draft_collaborator_ids.set(collaborator_ids.get());
        set_editing_collaborators.set(false);
    };

    let toggle_edit_mode = move || {
        let next = !edit_mode.get();
        set_edit_mode.set(next);
        if !next {
            cancel_edit_title();
            cancel_edit_description();
            cancel_edit_tags();
            cancel_edit_platforms();
            cancel_edit_extended();
            cancel_edit_collaborators();
        }
    };

    let commit_edit_title = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_title = set_title;
            let set_draft_title = set_draft_title;
            let set_editing_title = set_editing_title;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_title) = update_project_title(&project_id, draft_value).await {
                    set_title.set(new_title.clone());
                    set_draft_title.set(new_title.clone());
                    modal_card.update(|opt| {
                        if let Some(card) = opt.as_mut() {
                            card.title = new_title.clone();
                        }
                    });
                    set_projects.update(|projects| {
                        for project in projects.iter_mut() {
                            if project.id == project_id {
                                project.title = new_title.clone();
                                break;
                            }
                        }
                    });
                }
                set_editing_title.set(false);
            });
        })
    };

    let commit_edit_description = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_description = set_description;
            let set_draft_description = set_draft_description;
            let set_editing_description = set_editing_description;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_description) =
                    update_project_description(&project_id, draft_value).await
                {
                    set_description.set(new_description.clone());
                    set_draft_description.set(new_description.clone());
                    modal_card.update(|opt| {
                        if let Some(card) = opt.as_mut() {
                            card.description = new_description.clone();
                        }
                    });
                    set_projects.update(|projects| {
                        for project in projects.iter_mut() {
                            if project.id == project_id {
                                project.description = new_description.clone();
                                break;
                            }
                        }
                    });
                }
                set_editing_description.set(false);
            });
        })
    };

    let commit_edit_extended = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_extended_desc = set_extended_desc;
            let set_draft_extended = set_draft_extended;
            let set_editing_extended = set_editing_extended;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_extended) =
                    update_project_extended_desc(&project_id, draft_value).await
                {
                    set_extended_desc.set(new_extended.clone());
                    set_draft_extended.set(new_extended.clone());
                    modal_card.update(|opt| {
                        if let Some(card) = opt.as_mut() {
                            card.extended_desc = new_extended.clone();
                        }
                    });
                    set_projects.update(|projects| {
                        for project in projects.iter_mut() {
                            if project.id == project_id {
                                project.extended_desc = new_extended.clone();
                                break;
                            }
                        }
                    });
                }
                set_editing_extended.set(false);
            });
        })
    };

    let commit_edit_tags = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: Vec<crate::metadata::tags::Tag>| {
            let project_id = project_id.clone();
            let set_tags = set_tags;
            let set_draft_tags = set_draft_tags;
            let set_editing_tags = set_editing_tags;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_tags) = update_project_tags(&project_id, draft_value).await {
                    set_tags.set(new_tags.clone());
                    set_draft_tags.set(new_tags.clone());
                    modal_card.update(|opt| {
                        if let Some(card) = opt.as_mut() {
                            card.tags = new_tags.clone();
                        }
                    });
                    set_projects.update(|projects| {
                        for project in projects.iter_mut() {
                            if project.id == project_id {
                                project.tags = new_tags.clone();
                                break;
                            }
                        }
                    });
                }
                set_editing_tags.set(false);
            });
        })
    };

    let commit_edit_platforms = {
        let project_id = project_id.clone();
        Callback::new(
            move |draft_value: Vec<crate::metadata::platforms::Platform>| {
                let project_id = project_id.clone();
                let set_supported_platforms = set_supported_platforms;
                let set_draft_platforms = set_draft_platforms;
                let set_editing_platforms = set_editing_platforms;
                let modal_card = modal.set_card;
                let set_projects = projects_ctx.set_projects;

                leptos::task::spawn_local(async move {
                    if let Some(new_platforms) =
                        update_project_platforms(&project_id, draft_value).await
                    {
                        set_supported_platforms.set(new_platforms.clone());
                        set_draft_platforms.set(new_platforms.clone());
                        modal_card.update(|opt| {
                            if let Some(card) = opt.as_mut() {
                                card.supported_platforms = new_platforms.clone();
                            }
                        });
                        set_projects.update(|projects| {
                            for project in projects.iter_mut() {
                                if project.id == project_id {
                                    project.supported_platforms = new_platforms.clone();
                                    break;
                                }
                            }
                        });
                    }
                    set_editing_platforms.set(false);
                });
            },
        )
    };

    let commit_edit_collaborators = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: Vec<String>| {
            let project_id = project_id.clone();
            let set_collaborator_ids = set_collaborator_ids;
            let set_draft_collaborator_ids = set_draft_collaborator_ids;
            let set_editing_collaborators = set_editing_collaborators;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_ids) = update_project_collaborators(&project_id, draft_value).await
                {
                    set_collaborator_ids.set(new_ids.clone());
                    set_draft_collaborator_ids.set(new_ids.clone());
                    modal_card.update(|opt| {
                        if let Some(card) = opt.as_mut() {
                            card.collaborator_ids = new_ids.clone();
                        }
                    });
                    set_projects.update(|projects| {
                        for project in projects.iter_mut() {
                            if project.id == project_id {
                                project.collaborator_ids = new_ids.clone();
                                break;
                            }
                        }
                    });
                }
                set_editing_collaborators.set(false);
            });
        })
    };

    let commit_edit_icon = {
        let project_id = project_id.clone();
        Callback::new(move |file: web_sys::File| {
            let project_id = project_id.clone();
            let set_icon_url = set_icon_url;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_icon) = upload_project_icon(&project_id, file).await {
                    set_icon_url.set(Some(new_icon.clone()));
                    modal_card.update(|opt| {
                        if let Some(card) = opt.as_mut() {
                            card.icon_url = Some(new_icon.clone());
                        }
                    });
                    set_projects.update(|projects| {
                        for project in projects.iter_mut() {
                            if project.id == project_id {
                                project.icon_url = Some(new_icon.clone());
                                break;
                            }
                        }
                    });
                }
            });
        })
    };

    let toggle_favorite_click = {
        let project_id = card.id.clone();
        let set_projects = projects_ctx.set_projects;
        let modal_set_card = modal.set_card;
        Callback::new(move |_| {
            let project_id = project_id.clone();
            leptos::task::spawn_local(async move {
                if let Some(updated) = ProjectsContext::toggle_favorite(&project_id).await {
                    let updated_for_modal = updated.clone();
                    set_projects.update(|projects| {
                        if let Some(project) =
                            projects.iter_mut().find(|project| project.id == updated.id)
                        {
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
        })
    };

    let is_favorited = Signal::derive({
        let project_id = project_id.clone();
        move || {
            projects_ctx
                .projects
                .get()
                .into_iter()
                .find(|project| project.id == project_id)
                .and_then(|project| {
                    current_user
                        .account
                        .get()
                        .map(|me| project.favorites.contains(&me.id))
                })
                .unwrap_or(false)
        }
    });
    let favorite_count = Signal::derive({
        let project_id = project_id.clone();
        move || {
            projects_ctx
                .projects
                .get()
                .into_iter()
                .find(|project| project.id == project_id)
                .map(|project| project.favorites.len())
                .unwrap_or(card.favorites.len())
        }
    });

    let toggle_tag = Callback::new(move |tag: crate::metadata::tags::Tag| {
        set_draft_tags.update(|tags| {
            if let Some(pos) = tags.iter().position(|t| *t == tag) {
                tags.remove(pos);
            } else {
                tags.push(tag);
            }
        });
    });

    let toggle_platform = Callback::new(move |platform: crate::metadata::platforms::Platform| {
        set_draft_platforms.update(|platforms| {
            if let Some(pos) = platforms.iter().position(|p| *p == platform) {
                platforms.remove(pos);
            } else {
                platforms.push(platform);
            }
        });
    });

    let add_collaborator = Callback::new(move |account_id: String| {
        set_draft_collaborator_ids.update(|ids| {
            if !ids.contains(&account_id) {
                ids.push(account_id);
            }
        });
    });

    let remove_collaborator = Callback::new(move |account_id: String| {
        set_draft_collaborator_ids.update(|ids| {
            if let Some(pos) = ids.iter().position(|id| id == &account_id) {
                ids.remove(pos);
            }
        });
    });

    let author_id = card.author_id.clone();
    let accounts = AccountsContext::use_context();

    let apply_filter = Callback::new(move |filter: String| {
        search.set_query.set(format!("#{filter}"));
        modal.close();
    });

    view! {
        <div class="flex flex-col h-full min-h-0 overflow-hidden gap-4">
            <div class="flex items-start gap-4 relative p-2 pr-3">
                <div class="relative flex-shrink-0">
                    <input
                        node_ref=icon_input_ref
                        type="file"
                        class="hidden"
                        accept="image/png,image/jpeg,image/webp"
                        on:change=move |ev| {
                            let input = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                            let Some(input) = input else {
                                return;
                            };
                            let Some(files) = input.files() else {
                                return;
                            };
                            let Some(file) = files.get(0).and_then(|blob| blob.dyn_into::<web_sys::File>().ok()) else {
                                return;
                            };
                            commit_edit_icon.run(file);
                            input.set_value("");
                        }
                    />
                    {move || {
                        view! {
                            <ProjectIconPicker
                                icon_url={move || icon_url.get()}
                                title=move || title.get()
                                editable={Signal::derive(move || is_editable.get() && edit_mode.get())}
                                on_click=move |_| {
                                    if let Some(input) = icon_input_ref.get() {
                                        input.click();
                                    }
                                }
                                class="w-16 h-16"
                            />
                        }
                            .into_any()
                    }}
                </div>
                <div class="min-w-0 flex-1 flex flex-col gap-1">
                    {move || {
                        if editing_title.get() {
                            view! {
                                <div class="space-y-2">
                                    <div class="flex items-center gap-2">
                                        <input
                                            class=move || {
                                                let at_max = draft_title.get().len() >= MAX_TITLE_LENGTH;
                                                format!(
                                                    "input input-sm input-bordered flex-1 text-base-content text-xl font-bold {}",
                                                    if at_max { "hover:border-error" } else { "" }
                                                )
                                            }
                                            type="text"
                                            maxlength=MAX_TITLE_LENGTH.to_string()
                                            prop:value=draft_title.get()
                                            on:input=move |ev| set_draft_title.set(event_target_value(&ev))
                                            on:keyup=move |ev| {
                                                match ev.key().as_str() {
                                                    "Enter" => commit_edit_title.run(draft_title.get()),
                                                    "Escape" => cancel_edit_title(),
                                                    _ => {}
                                                }
                                            }
                                            autofocus
                                        />
                                        <span class=move || {
                                            if draft_title.get().len() >= MAX_TITLE_LENGTH {
                                                "text-xs text-error flex-shrink-0"
                                            } else {
                                                "text-xs text-base-content/50 flex-shrink-0"
                                            }
                                        }>
                                            {move || format!("{}/{}", draft_title.get().len(), MAX_TITLE_LENGTH)}
                                        </span>
                                    </div>
                                    <div class="flex justify-end gap-2">
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs"
                                            on:click=move |_| cancel_edit_title()
                                        >"Cancel"</button>
                                        <button
                                            type="button"
                                            class="btn btn-primary btn-xs"
                                            on:click=move |_| commit_edit_title.run(draft_title.get())
                                        >"Save"</button>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="flex items-center gap-2">
                                    <h2
                                        class="text-xl font-bold text-primary leading-tight truncate tooltip tooltip-top"
                                        data-tip={title.get()}
                                    >
                                        {title.get()}
                                        </h2>
                                        {move || (is_editable.get() && edit_mode.get()).then(|| view! {
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary"
                                            aria-label="Edit title"
                                            on:click=move |_| start_edit_title()
                                        >
                                            {edit_pencil_icon("w-4 h-4")}
                                        </button>
                                    }.into_any())}
                                </div>
                            }
                                .into_any()
                        }
                    }}
                    {move || {
                        if editing_description.get() {
                            view! {
                                <div class="space-y-2">
                                    <textarea
                                        class=move || {
                                            let at_max = draft_description.get().len() >= MAX_DESCRIPTION_LENGTH;
                                            format!(
                                                "textarea w-full min-h-[5rem] rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none {}",
                                                if at_max { "hover:border-error" } else { "" }
                                            )
                                        }
                                        maxlength=MAX_DESCRIPTION_LENGTH.to_string()
                                        prop:value=draft_description.get()
                                        on:input=move |ev| set_draft_description.set(event_target_value(&ev))
                                        on:keyup=move |ev| {
                                            if ev.key().as_str() == "Escape" {
                                                cancel_edit_description();
                                            }
                                        }
                                        autofocus
                                    ></textarea>
                                    <div class="flex items-center justify-between">
                                        <span class=move || {
                                            if draft_description.get().len() >= MAX_DESCRIPTION_LENGTH {
                                                "text-xs text-error"
                                            } else {
                                                "text-xs text-base-content/50"
                                            }
                                        }>
                                            {move || format!("{}/{}", draft_description.get().len(), MAX_DESCRIPTION_LENGTH)}
                                        </span>
                                        <div class="flex gap-2">
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-xs"
                                                on:click=move |_| cancel_edit_description()
                                            >"Cancel"</button>
                                            <button
                                                type="button"
                                                class="btn btn-primary btn-xs"
                                                on:click=move |_| commit_edit_description.run(draft_description.get())
                                            >"Save"</button>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="flex items-center gap-2">
                                    <p class="text-base-content/70 text-sm whitespace-pre-wrap">{description.get()}</p>
                                    {move || (is_editable.get() && edit_mode.get()).then(|| view! {
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary flex-shrink-0"
                                            aria-label="Edit description"
                                            on:click=move |_| start_edit_description()
                                        >
                                            {edit_pencil_icon("w-4 h-4")}
                                        </button>
                                    }.into_any())}
                                </div>
                            }
                                .into_any()
                        }
                    }}
                </div>
                <div class="hidden sm:flex items-center gap-2 text-xs flex-shrink-0">
                    <button
                        type="button"
                        class=move || {
                            if is_favorited.get() {
                                "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-error hover:text-base-content/50 tooltip tooltip-bottom"
                            } else {
                                "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-error tooltip tooltip-bottom"
                            }
                        }
                        aria-label=move || {
                            if is_favorited.get() {
                                format!("Remove {} from favorites", title.get())
                            } else {
                                format!("Add {} to favorites", title.get())
                            }
                        }
                        data-tip=move || {
                            if is_favorited.get() {
                                "Remove favorite".to_string()
                            } else {
                                "Add favorite".to_string()
                            }
                        }
                        on:click={
                            let cb = toggle_favorite_click;
                            move |_| cb.run(())
                        }
                    >
                        <HeartIcon filled=Signal::derive(move || is_favorited.get()) />
                        <span>{move || favorite_count.get().to_string()}</span>
                    </button>
                    {move || is_editable.get().then(|| view! {
                    <button
                        type="button"
                        class=move || {
                            if edit_mode.get() {
                                "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-primary tooltip tooltip-bottom"
                            } else {
                                "btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary tooltip tooltip-bottom"
                            }
                        }
                        aria-label=move || if edit_mode.get() { "Leave edit mode" } else { "Enter edit mode" }
                        data-tip=move || if edit_mode.get() { "Done editing" } else { "Edit project" }
                        on:click=move |_| toggle_edit_mode()
                    >
                        {edit_pencil_icon("w-4 h-4")}
                    </button>
                }.into_any())}
                <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">esc</kbd>
                    <span class="text-base-content/50">to close</span>
                </div>
            </div>

            <hr class="border-base-content/10" />

            <div class="flex flex-col min-h-0 overflow-hidden flex-1 py-2">
                <div class="flex items-center justify-between gap-3 pb-2 flex-shrink-0">
                    <div class="tabs tabs-border">
                        <button
                            type="button"
                            class=move || if active_tab.get() == ProjectDetailsTab::About { "tab tab-active" } else { "tab" }
                            on:click=move |_| set_active_tab.set(ProjectDetailsTab::About)
                        >"About"</button>
                        <button
                            type="button"
                            class=move || if active_tab.get() == ProjectDetailsTab::Viewer3d { "tab tab-active" } else { "tab" }
                            on:click=move |_| set_active_tab.set(ProjectDetailsTab::Viewer3d)
                        >"3D viewer"</button>
                        <button
                            type="button"
                            class=move || if active_tab.get() == ProjectDetailsTab::Versions { "tab tab-active" } else { "tab" }
                            on:click=move |_| set_active_tab.set(ProjectDetailsTab::Versions)
                        >"Versions"</button>
                    </div>
                </div>
                <div class="overflow-y-auto flex-1 min-h-0 p-2 pr-3 space-y-4">
                    <div class="grid grid-cols-1 xl:grid-cols-[minmax(0,2fr)_1px_minmax(18rem,1fr)] gap-6 items-start">
                        <div class="min-w-0 space-y-4">
                            {move || match active_tab.get() {
                                ProjectDetailsTab::About => {
                                    if editing_extended.get() {
                                        view! {
                                            <MarkdownEditor
                                                value=draft_extended
                                                on_input=Callback::new(move |value| set_draft_extended.set(value))
                                                on_cancel=Callback::new(move |_| cancel_edit_extended())
                                                on_save=Callback::new(move |_| commit_edit_extended.run(draft_extended.get()))
                                                maxlength=MAX_EXTENDED_DESC_LENGTH
                                                editor_class="min-h-[20rem] font-mono text-sm"
                                            />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            {move || {
                                                if is_editable.get() && edit_mode.get() {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="group relative text-left w-full min-h-[20rem] rounded-none border border-base-content/10 bg-base-200/20 p-4 overflow-auto hover:border-primary transition-colors cursor-pointer"
                                                            aria-label="Edit extended description"
                                                            on:click=move |_| start_edit_extended()
                                                        >
                                                            <MarkdownView source=extended_desc.get() />
                                                            <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                {edit_pencil_icon("w-5 h-5 text-primary")}
                                                            </div>
                                                        </button>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <div class="min-h-[20rem] rounded-none border border-base-content/10 bg-base-200/20 p-4 overflow-auto">
                                                            <MarkdownView source=extended_desc.get() />
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        }
                                            .into_any()
                                    }
                                }
                                ProjectDetailsTab::Viewer3d => view! {
                                    <div class="min-h-[20rem] rounded-none border border-base-content/10 bg-base-200/20 p-4 flex items-center justify-center text-base-content/50 text-sm">
                                        "3D viewer coming later."
                                    </div>
                                }
                                    .into_any(),
                                ProjectDetailsTab::Versions => view! {
                                    <div class="min-h-[20rem] rounded-none border border-base-content/10 bg-base-200/20 p-4 flex items-center justify-center text-base-content/50 text-sm">
                                        "Versions coming later."
                                    </div>
                                }
                                    .into_any(),
                            }}
                        </div>

                        <div class="hidden xl:block self-stretch w-px bg-base-content/10" aria-hidden="true"></div>

                        <div class="space-y-4">
                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4">
                                <h3 class="text-sm font-semibold text-base-content mb-3">"Supported platforms"</h3>
                                {move || {
                                    if editing_platforms.get() {
                                        view! {
                                            <EditableChipSection
                                                title="Supported platforms"
                                                aria_label="Supported platforms"
                                                items=supported_platforms.get()
                                                all_items=crate::metadata::platforms::Platform::all().to_vec()
                                                editing=editing_platforms.into()
                                                on_cancel=Callback::new(move |_| cancel_edit_platforms())
                                                on_toggle=toggle_platform
                                                on_save=Callback::new(move |selected| commit_edit_platforms.run(selected))
                                                on_item_click=Callback::new(move |platform: crate::metadata::platforms::Platform| apply_filter.run(platform.label().to_string()))
                                                label_fn=crate::metadata::platforms::platform_label
                                                color_fn=crate::metadata::platforms::platform_color
                                                selected_items=draft_platforms.into()
                                                badge_class="badge badge-sm badge-outline rounded-none border-base-content/10 whitespace-nowrap hover:border-primary/40 cursor-pointer"
                                            />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            {move || {
                                                if is_editable.get() && edit_mode.get() {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="group relative text-left w-full border border-base-content/20 rounded-none p-2 hover:border-primary transition-colors cursor-pointer"
                                                            aria-label="Edit supported platforms"
                                                            on:click=move |_| start_edit_platforms()
                                                        >
                                                            <div class="flex flex-wrap gap-2" role="group" aria-label="Supported platforms">
                                                                {supported_platforms.get().iter().map(|item| {
                                                                    view! {
                                                                        <span class=format!(
                                                                            "badge badge-sm badge-outline rounded-none border-base-content/10 whitespace-nowrap {}",
                                                                            crate::metadata::platforms::platform_color(item)
                                                                        )>
                                                                            {crate::metadata::platforms::platform_label(item)}
                                                                        </span>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                            <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                {edit_pencil_icon("w-5 h-5 text-primary")}
                                                            </div>
                                                        </button>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <div class="flex flex-wrap gap-2" role="group" aria-label="Supported platforms">
                                                            {supported_platforms.get().iter().map(|item| {
                                                                let item_for_click = *item;
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        class=format!(
                                                                            "badge badge-sm badge-outline rounded-none border-base-content/10 whitespace-nowrap hover:border-primary/40 cursor-pointer {}",
                                                                            crate::metadata::platforms::platform_color(item)
                                                                        )
                                                                        on:click=move |_| apply_filter.run(crate::metadata::platforms::platform_label(&item_for_click).to_string())
                                                                    >
                                                                        {crate::metadata::platforms::platform_label(item)}
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>

                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4">
                                <h3 class="text-sm font-semibold text-base-content mb-3">"Tags"</h3>
                                {move || {
                                    if editing_tags.get() {
                                        view! {
                                            <EditableChipSection
                                                title="Tags"
                                                aria_label="Tags"
                                                items=tags.get()
                                                all_items=crate::metadata::tags::Tag::all().to_vec()
                                                editing=editing_tags.into()
                                                on_cancel=Callback::new(move |_| cancel_edit_tags())
                                                on_toggle=toggle_tag
                                                on_save=Callback::new(move |selected| commit_edit_tags.run(selected))
                                                on_item_click=Callback::new(move |tag: crate::metadata::tags::Tag| apply_filter.run(tag.label().to_string()))
                                                label_fn=crate::metadata::tags::tag_label
                                                color_fn=crate::metadata::tags::tag_color
                                                selected_items=draft_tags.into()
                                                badge_class="badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap hover:border-primary/40 cursor-pointer"
                                            />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            {move || {
                                                if is_editable.get() && edit_mode.get() {
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="group relative text-left w-full border border-base-content/20 rounded-none p-2 hover:border-primary transition-colors cursor-pointer"
                                                            aria-label="Edit tags"
                                                            on:click=move |_| start_edit_tags()
                                                        >
                                                            <div class="flex flex-wrap gap-2" role="group" aria-label="Tags">
                                                                {tags.get().iter().map(|item| {
                                                                    view! {
                                                                        <span class=format!(
                                                                            "badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap {}",
                                                                            crate::metadata::tags::tag_color(item)
                                                                        )>
                                                                            {crate::metadata::tags::tag_label(item)}
                                                                        </span>
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                            <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                {edit_pencil_icon("w-5 h-5 text-primary")}
                                                            </div>
                                                        </button>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <div class="flex flex-wrap gap-2" role="group" aria-label="Tags">
                                                            {tags.get().iter().map(|item| {
                                                                let item_for_click = *item;
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        class=format!(
                                                                            "badge badge-sm badge-outline rounded-none text-neutral-900 border-base-content/10 whitespace-nowrap hover:border-primary/40 cursor-pointer {}",
                                                                            crate::metadata::tags::tag_color(item)
                                                                        )
                                                                        on:click=move |_| apply_filter.run(crate::metadata::tags::tag_label(&item_for_click).to_string())
                                                                    >
                                                                        {crate::metadata::tags::tag_label(item)}
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>

                            <div class="rounded-none border border-base-content/10 bg-base-200/20 p-4 space-y-3">
                                <h3 class="text-sm font-semibold text-base-content">"Authors"</h3>
                                {move || {
                                    let all_accounts = accounts.accounts.get();
                                    let author_id = author_id.clone();

                                    if editing_collaborators.get() {
                                        let owner_account = all_accounts.iter().find(|account| account.id == author_id).cloned();
                                        let all_accounts_for_select = all_accounts.clone();
                                        let (add_open, set_add_open) = signal(false);
                                        let add_open_signal = Signal::derive(move || add_open.get());
                                        let draft_query = RwSignal::new(String::new());
                                        let selectable_accounts = Memo::new(move |_| {
                                            let query = draft_query.get().to_lowercase();
                                            let excluded_ids: std::collections::HashSet<String> = std::iter::once(author_id.clone())
                                                .chain(draft_collaborator_ids.get().into_iter())
                                                .collect();
                                            all_accounts_for_select
                                                .clone()
                                                .into_iter()
                                                .filter(|account| !excluded_ids.contains(&account.id))
                                                .filter(|account| {
                                                    query.is_empty()
                                                        || account.username.to_lowercase().contains(&query)
                                                        || account.display_name.to_lowercase().contains(&query)
                                                })
                                                .collect::<Vec<_>>()
                                        });

                                        view! {
                                            <div class="space-y-3">
                                                <div class="flex flex-wrap gap-2 items-center">
                                                    {owner_account.as_ref().map(|account| {
                                                        let account = account.clone();
                                                        view! {
                                                            {avatar_button(&account, None)}
                                                        }
                                                    })}
                                                    {draft_collaborator_ids.get().into_iter().filter_map(|id| {
                                                        let all_accounts = all_accounts.clone();
                                                        all_accounts.iter().find(|account| account.id == id).cloned()
                                                    }).map(|account| {
                                                        let account_id = account.id.clone();
                                                        let display_name = account.display_name.clone();
                                                        view! {
                                                            <div class="relative group">
                                                                {avatar_button(&account, None)}
                                                                <button
                                                                    type="button"
                                                                    class="absolute inset-0 flex items-center justify-center bg-error/80 opacity-0 group-hover:opacity-100 transition-opacity text-white tooltip tooltip-top"
                                                                    data-tip={format!("Remove {}", display_name)}
                                                                    aria-label={format!("Remove {}", display_name)}
                                                                    on:click=move |_| remove_collaborator.run(account_id.clone())
                                                                >
                                                                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                                        <line x1="5" y1="12" x2="19" y2="12" />
                                                                    </svg>
                                                                </button>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                    <button
                                                        type="button"
                                                        class="w-12 h-12 border border-dashed border-base-content/30 flex items-center justify-center text-base-content/50 hover:border-primary hover:text-primary transition-colors tooltip tooltip-top"
                                                        aria-label="Add collaborator"
                                                        data-tip="Add collaborator"
                                                        on:click=move |_| set_add_open.set(true)
                                                    >
                                                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                                            <line x1="12" y1="5" x2="12" y2="19" />
                                                            <line x1="5" y1="12" x2="19" y2="12" />
                                                        </svg>
                                                    </button>
                                                </div>
                                                <div class="flex justify-end gap-2">
                                                    <button
                                                        type="button"
                                                        class="btn btn-ghost btn-xs"
                                                        on:click=move |_| cancel_edit_collaborators()
                                                    >"Cancel"</button>
                                                    <button
                                                        type="button"
                                                        class="btn btn-primary btn-xs"
                                                        on:click=move |_| commit_edit_collaborators.run(draft_collaborator_ids.get())
                                                    >"Save"</button>
                                                </div>
                                                <SearchModal
                                                    open=add_open_signal
                                                    on_close=Callback::new(move |_| set_add_open.set(false))
                                                >
                                                    <div class="space-y-3">
                                                        <h3 class="text-sm font-semibold text-base-content">"Add collaborator"</h3>
                                                        <input
                                                            type="text"
                                                            class="input w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none"
                                                            placeholder="Search users..."
                                                            prop:value=draft_query.get()
                                                            on:input=move |ev| draft_query.set(event_target_value(&ev))
                                                        />
                                                        <div class="max-h-60 overflow-y-auto space-y-1">
                                                            {move || {
                                                                let accounts = selectable_accounts.get();
                                                                if accounts.is_empty() {
                                                                    view! {
                                                                        <p class="text-sm text-error py-2">"No users found."</p>
                                                                    }.into_any()
                                                                } else {
                                                                    view! {
                                                                        <div class="flex flex-wrap gap-2">
                                                                            {accounts.into_iter().map(|account| {
                                                                                let account_id = account.id.clone();
                                                                                let display_name = account.display_name.clone();
                                                                                view! {
                                                                                    <button
                                                                                        type="button"
                                                                                        class="flex items-center gap-2 px-2 py-1 border border-base-content/10 hover:border-primary/40 transition-colors"
                                                                                        on:click=move |_| {
                                                                                            add_collaborator.run(account_id.clone());
                                                                                            set_add_open.set(false);
                                                                                        }
                                                                                    >
                                                                                        {avatar_button(&account, Some("w-8 h-8".to_string()))}
                                                                                        <span class="text-sm text-base-content">{display_name.clone()}</span>
                                                                                        <span class="text-xs text-base-content/50">{format!("@{}", account.username)}</span>
                                                                                    </button>
                                                                                }
                                                                            }).collect_view()}
                                                                        </div>
                                                                    }.into_any()
                                                                }
                                                            }}
                                                        </div>
                                                    </div>
                                                </SearchModal>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            {move || {
                                                let all_accounts = accounts.accounts.get();
                                                let author_id = author_id.clone();
                                                let owner_account = all_accounts.iter().find(|account| account.id == author_id).cloned();
                                                let current_collaborators = all_accounts
                                                    .iter()
                                                    .filter(|account| collaborator_ids.get().contains(&account.id))
                                                    .cloned()
                                                    .collect::<Vec<_>>();

                                                if is_editable.get() && edit_mode.get() {
                                                    let collaborators = current_collaborators.clone();
                                                    view! {
                                                        <button
                                                            type="button"
                                                            class="group relative text-left w-full border border-base-content/20 rounded-none p-2 hover:border-primary transition-colors cursor-pointer"
                                                            aria-label="Edit collaborators"
                                                            on:click=move |_| start_edit_collaborators()
                                                        >
                                                            <div class="flex flex-wrap gap-2">
                                                                {owner_account.as_ref().map(|account| {
                                                                    let account = account.clone();
                                                                    view! {
                                                                        {avatar_button(&account, None)}
                                                                    }
                                                                })}
                                                                {collaborators.into_iter().map(|account| {
                                                                    view! {
                                                                        {avatar_button(&account, None)}
                                                                    }
                                                                }).collect_view()}
                                                            </div>
                                                            <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                                {edit_pencil_icon("w-5 h-5 text-primary")}
                                                            </div>
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    let collaborators = current_collaborators.clone();
                                                    view! {
                                                        <div class="flex flex-wrap gap-2">
                                                            {owner_account.as_ref().map(|account| {
                                                                let account_for_click = account.clone();
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |_| {
                                                                            profile_modal.open(account_for_click.clone());
                                                                        }
                                                                    >
                                                                        {avatar_button(account, None)}
                                                                    </button>
                                                                }
                                                            })}
                                                            {collaborators.into_iter().map(|account| {
                                                                let account_for_click = account.clone();
                                                                view! {
                                                                    <button
                                                                        type="button"
                                                                        on:click=move |_| {
                                                                            profile_modal.open(account_for_click.clone());
                                                                        }
                                                                    >
                                                                        {avatar_button(&account, None)}
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>

                    {move || {
                        if is_editable.get() && edit_mode.get() {
                            view! {
                                <div class="border border-error/30 bg-error/10 p-4 space-y-3">
                                    <div class="flex items-start gap-3">
                                        {warning_icon("w-5 h-5 text-error flex-shrink-0 mt-0.5")}
                                        <div class="flex-1 min-w-0">
                                            <p class="text-sm font-semibold text-error">{"Danger zone"}</p>
                                            <p class="text-sm text-base-content/80">
                                                {"Deleting this project cannot be undone. Type the project name below to confirm."}
                                            </p>
                                        </div>
                                    </div>

                                    {move || {
                                        if show_delete_confirm.get() {
                                            view! {
                                                <div class="space-y-3">
                                                    <label class="block text-sm text-base-content" for="delete-confirm-input">
                                                        {"Type "}
                                                        <span class="font-semibold text-error">{title.get()}</span>
                                                        {" to confirm"}
                                                    </label>
                                                    <input
                                                        id="delete-confirm-input"
                                                        type="text"
                                                        class="input w-full rounded-none bg-transparent border-base-content/20 focus:border-error focus:outline-none"
                                                        placeholder={title.get()}
                                                        prop:value=delete_confirm_input.get()
                                                        on:input=move |ev| set_delete_confirm_input.set(event_target_value(&ev))
                                                        disabled=move || is_deleting.get()
                                                    />
                                                    <div class="flex justify-end gap-2">
                                                        <button
                                                            type="button"
                                                            class="btn btn-ghost btn-xs"
                                                            on:click=move |_| {
                                                                set_show_delete_confirm.set(false);
                                                                set_delete_confirm_input.set(String::new());
                                                            }
                                                            disabled=move || is_deleting.get()
                                                        >
                                                            {"Cancel"}
                                                        </button>
                                                        <button
                                                            type="button"
                                                            class="btn btn-error btn-xs"
                                                            on:click=move |_| delete_project_click.run(())
                                                            disabled=move || !can_delete.get() || is_deleting.get()
                                                        >
                                                            {move || if is_deleting.get() {
                                                                view! {
                                                                    <span class="flex items-center gap-2">
                                                                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                                                        <span>{"Deleting..."}</span>
                                                                    </span>
                                                                }
                                                                    .into_any()
                                                            } else {
                                                                view! {
                                                                    <span class="flex items-center gap-1">
                                                                        {trash_icon("w-3.5 h-3.5")}
                                                                        <span>{"Delete project"}</span>
                                                                    </span>
                                                                }
                                                                    .into_any()
                                                            }}
                                                        </button>
                                                    </div>
                                                </div>
                                            }
                                                .into_any()
                                        } else {
                                            view! {
                                                <button
                                                    type="button"
                                                    class="btn btn-outline btn-error btn-xs"
                                                    on:click=move |_| set_show_delete_confirm.set(true)
                                                >
                                                    <span class="flex items-center gap-1">
                                                        {trash_icon("w-3.5 h-3.5")}
                                                        <span>{"Delete project"}</span>
                                                    </span>
                                                </button>
                                            }
                                                .into_any()
                                        }
                                    }}
                                </div>
                            }
                                .into_any()
                        } else {
                            ().into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

fn event_target_value(ev: &leptos::web_sys::Event) -> String {
    ev.target()
        .and_then(|t| {
            t.dyn_into::<leptos::web_sys::HtmlTextAreaElement>()
                .ok()
                .map(|textarea| textarea.value())
        })
        .or_else(|| {
            ev.target().and_then(|t| {
                t.dyn_into::<leptos::web_sys::HtmlInputElement>()
                    .ok()
                    .map(|input| input.value())
            })
        })
        .unwrap_or_default()
}
