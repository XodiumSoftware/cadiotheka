use crate::components::cards::project::ProjectCardProperties;
use crate::components::ui::markdown::MarkdownView;
use crate::components::ui::modals::search::SearchModal;
use crate::components::ui::overflow_row::{OverflowItem, OverflowRow};
use crate::components::ui::project_icon_picker::ProjectIconPicker;
use crate::contexts::{
    AccountsContext, CurrentUserContext, ProfileModalContext, ProjectModalContext, ProjectsContext,
};
use crate::data::{
    AccountRole, update_project_extended_desc, update_project_title, upload_project_icon,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

const MAX_TITLE_LENGTH: usize = 100;
const MAX_EXTENDED_DESC_LENGTH: usize = 5000;

/// Modal dialog that displays detailed information about a selected project.
#[component]
pub fn ProjectModal() -> impl IntoView {
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
                        <p class="text-base-content/50 text-sm">No project selected.</p>
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
    let current_user = CurrentUserContext::use_context();
    let projects_ctx = ProjectsContext::use_context();
    let modal = ProjectModalContext::use_context();
    let is_editable = current_user
        .account
        .get()
        .is_some_and(|me| me.role == AccountRole::Admin || me.id == card.author_id);

    let (editing, set_editing) = signal(false);
    let (draft, set_draft) = signal(card.title.clone());
    let (title, set_title) = signal(card.title.clone());
    let (editing_icon, set_editing_icon) = signal(false);
    let (selected_icon_file, set_selected_icon_file) = signal(Option::<web_sys::File>::None);
    let (icon_url, set_icon_url) = signal(card.icon_url.clone());
    let (editing_extended, set_editing_extended) = signal(false);
    let (draft_extended, set_draft_extended) = signal(card.extended_desc.clone());
    let (extended_desc, set_extended_desc) = signal(card.extended_desc.clone());
    let project_id = card.id.clone();

    let start_edit = move |_| {
        set_draft.set(title.get());
        set_editing.set(true);
    };

    let cancel_edit = move || {
        set_editing.set(false);
    };

    let cancel_edit_icon = move || {
        set_selected_icon_file.set(None);
        set_editing_icon.set(false);
    };

    let start_edit_extended = move || {
        set_draft_extended.set(extended_desc.get());
        set_editing_extended.set(true);
    };

    let cancel_edit_extended = move || {
        set_editing_extended.set(false);
    };

    let commit_edit_icon = {
        let project_id = project_id.clone();
        Callback::new(move |file: web_sys::File| {
            let project_id = project_id.clone();
            let set_icon_url = set_icon_url;
            let set_editing_icon = set_editing_icon;
            let set_selected_icon_file = set_selected_icon_file;
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
                set_selected_icon_file.set(None);
                set_editing_icon.set(false);
            });
        })
    };

    let commit_edit_extended = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_extended_desc = set_extended_desc;
            let set_editing_extended = set_editing_extended;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_extended) =
                    update_project_extended_desc(&project_id, draft_value).await
                {
                    set_extended_desc.set(new_extended.clone());
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

    let commit_edit = {
        let project_id = project_id.clone();
        Callback::new(move |draft_value: String| {
            let project_id = project_id.clone();
            let set_title = set_title;
            let set_editing = set_editing;
            let modal_card = modal.set_card;
            let set_projects = projects_ctx.set_projects;

            leptos::task::spawn_local(async move {
                if let Some(new_title) = update_project_title(&project_id, draft_value).await {
                    set_title.set(new_title.clone());
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
                set_editing.set(false);
            });
        })
    };

    let author = card.author.clone();
    let author_username = card.author_username.clone();
    let author_id = card.author_id.clone();
    let accounts = AccountsContext::use_context();
    let open_author_profile = {
        let author_id = author_id.clone();
        move |_| {
            on_close.run(());
            let account = accounts
                .accounts
                .get()
                .into_iter()
                .find(|a| a.id == author_id);
            if let Some(account) = account {
                ProfileModalContext::use_context().open(account);
            }
        }
    };
    let tags = card.tags.clone();
    let platforms = card.supported_platforms.clone();

    view! {
        <div class="space-y-4 flex flex-col min-h-0">
            <div class="flex items-start gap-4">
                {move || {
                    view! {
                        <ProjectIconPicker
                            icon_url={move || icon_url.get()}
                            title=move || title.get()
                            editable={Signal::derive(move || is_editable)}
                            on_click=move |_| {
                                set_selected_icon_file.set(None);
                                set_editing_icon.update(|v| *v = !*v);
                            }
                            class="w-16 h-16"
                        />
                    }
                        .into_any()
                }}
                <div class="min-w-0 flex-1 flex flex-col gap-1">
                    {move || {
                        if editing_icon.get() {
                            view! {
                                <div class="flex items-center gap-2">
                                    <input
                                        class="file-input file-input-bordered flex-1 text-base-content"
                                        type="file"
                                        accept="image/png,image/jpeg,image/webp"
                                        on:change=move |ev| {
                                            let input = ev.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
                                            let Some(input) = input else {
                                                return;
                                            };
                                            let Some(files) = input.files() else {
                                                set_selected_icon_file.set(None);
                                                return;
                                            };
                                            let Some(file) = files.get(0).and_then(|blob| blob.dyn_into::<web_sys::File>().ok()) else {
                                                set_selected_icon_file.set(None);
                                                return;
                                            };
                                            set_selected_icon_file.set(Some(file));
                                        }
                                        autofocus
                                    />
                                    <div class="flex gap-2">
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs"
                                            on:click=move |_| cancel_edit_icon()
                                        >"Cancel"</button>
                                        <button
                                            type="button"
                                            class="btn btn-primary btn-xs"
                                            on:click=move |_| {
                                                if let Some(file) = selected_icon_file.get() {
                                                    commit_edit_icon.run(file);
                                                }
                                            }
                                        >"Upload"</button>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else if editing.get() {
                            view! {
                                <div class="flex items-center gap-2">
                                    <input
                                        class="input input-sm input-bordered flex-1 text-base-content text-xl font-bold"
                                        type="text"
                                        maxlength=MAX_TITLE_LENGTH.to_string()
                                        prop:value=draft.get()
                                        on:input=move |ev| set_draft.set(event_target_value(&ev))
                                        on:keyup=move |ev| {
                                            match ev.key().as_str() {
                                                "Enter" => commit_edit.run(draft.get()),
                                                "Escape" => cancel_edit(),
                                                _ => {}
                                            }
                                        }
                                        autofocus
                                    />
                                    <span class="text-xs text-base-content/50 flex-shrink-0">
                                        {move || format!("{}/{}", draft.get().len(), MAX_TITLE_LENGTH)}
                                    </span>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="flex items-center gap-2">
                                    <h2
                                        class="text-xl font-bold text-primary leading-tight truncate"
                                        title={title.get()}
                                    >
                                        {title.get()}
                                    </h2>
                                    {is_editable.then(|| view! {
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs p-1 h-auto min-h-0"
                                            aria-label="Edit project title"
                                            on:click=start_edit
                                        >
                                            <svg
                                                class="w-4 h-4 text-base-content/60"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                            >
                                                <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                                            </svg>
                                        </button>
                                    })}
                                </div>
                            }
                                .into_any()
                        }
                    }}
                    <p class="text-base-content/70 text-sm">
                        by
                        <button
                            type="button"
                            class="font-semibold text-base-content ml-1 hover:text-primary hover:underline"
                            title={format!("@{}", author_username)}
                            on:click=open_author_profile
                        >
                            {author.clone()}
                        </button>
                    </p>
                </div>
                <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50 flex-shrink-0">
                    <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">esc</kbd>
                    <span>to close</span>
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
                    <div class="flex items-center justify-between mb-1">
                        <h3 class="text-sm font-semibold text-base-content">"Description"</h3>
                        {is_editable.then(|| view! {
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs p-1 h-auto min-h-0"
                                aria-label="Edit extended description"
                                on:click=move |_| start_edit_extended()
                            >
                                <svg
                                    class="w-4 h-4 text-base-content/60"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                >
                                    <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                                </svg>
                            </button>
                        })}
                    </div>
                    {move || {
                        if editing_extended.get() {
                            view! {
                                <div class="space-y-2">
                                    <textarea
                                        class="textarea w-full min-h-[8rem] rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none"
                                        maxlength=MAX_EXTENDED_DESC_LENGTH.to_string()
                                        prop:value=draft_extended.get()
                                        on:input=move |ev| set_draft_extended.set(event_target_value(&ev))
                                        on:keyup=move |ev| {
                                            if ev.key().as_str() == "Escape" {
                                                cancel_edit_extended();
                                            }
                                        }
                                        autofocus
                                    ></textarea>
                                    <div class="flex items-center justify-between">
                                        <span class="text-xs text-base-content/50">
                                            {move || format!("{}/{}", draft_extended.get().len(), MAX_EXTENDED_DESC_LENGTH)}
                                        </span>
                                        <div class="flex gap-2">
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-xs"
                                                on:click=move |_| cancel_edit_extended()
                                            >"Cancel"</button>
                                            <button
                                                type="button"
                                                class="btn btn-primary btn-xs"
                                                on:click=move |_| commit_edit_extended.run(draft_extended.get())
                                            >"Save"</button>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <MarkdownView source=extended_desc.get() />
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>

            <hr class="border-base-content/10" />
        </div>
    }
}
