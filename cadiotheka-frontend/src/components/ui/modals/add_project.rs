use crate::components::ui::modals::search::SearchModal;
use crate::contexts::{
    AddProjectModalContext, CurrentUserContext, LoginModalContext, ProjectsContext,
};
use crate::data::{create_project, new_project_payload};
use crate::i18n::{t_string, use_i18n};
use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

/// Client-side validation result for the add-project form.
#[derive(Debug, Default, Clone)]
struct FormErrors {
    title: Option<String>,
    description: Option<String>,
    tags: Option<String>,
    platforms: Option<String>,
}

impl FormErrors {
    fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.tags.is_none()
            && self.platforms.is_none()
    }
}

/// Modal dialog for adding a new project.
#[component]
pub fn AddProjectModal() -> impl IntoView {
    let i18n = use_i18n();
    let modal = AddProjectModalContext::use_context();
    let current_user = CurrentUserContext::use_context();
    let login_modal = LoginModalContext::use_context();
    let projects_ctx = ProjectsContext::use_context();
    let on_close = move |_| modal.close();

    // Form fields
    let title_input_ref: NodeRef<leptos::html::Input> = NodeRef::new();
    let desc_input_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let extended_input_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();
    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (extended_desc, set_extended_desc) = signal(String::new());
    let (selected_tags, set_selected_tags) = signal(Vec::<Tag>::new());
    let (selected_platforms, set_selected_platforms) = signal(Vec::<Platform>::new());
    let (errors, set_errors) = signal(FormErrors::default());
    let (is_submitting, set_is_submitting) = signal(false);
    let (submit_error, set_submit_error) = signal(Option::<String>::None);

    let reset_form = move || {
        set_title.set(String::new());
        set_description.set(String::new());
        set_extended_desc.set(String::new());
        set_selected_tags.set(Vec::new());
        set_selected_platforms.set(Vec::new());
        set_errors.set(FormErrors::default());
        set_submit_error.set(None);
        if let Some(input) = title_input_ref.get() {
            input.set_value("");
        }
        if let Some(input) = desc_input_ref.get() {
            input.set_value("");
        }
        if let Some(input) = extended_input_ref.get() {
            input.set_value("");
        }
    };

    // Reset the form whenever the modal opens so stale data isn't shown.
    Effect::new(move |_| {
        if modal.open.get() {
            reset_form();
        }
    });

    let validate = move || {
        let mut e = FormErrors::default();
        let t = title.get();
        if t.trim().is_empty() {
            e.title = Some(t_string!(i18n, add_project.error_title_required).to_string());
        } else if t.trim().len() > 120 {
            e.title = Some(t_string!(i18n, add_project.error_title_long).to_string());
        }

        let d = description.get();
        if d.trim().is_empty() {
            e.description =
                Some(t_string!(i18n, add_project.error_description_required).to_string());
        } else if d.trim().len() > 500 {
            e.description = Some(t_string!(i18n, add_project.error_description_long).to_string());
        }

        if selected_tags.get().is_empty() {
            e.tags = Some(t_string!(i18n, add_project.error_tags_required).to_string());
        }

        if selected_platforms.get().is_empty() {
            e.platforms = Some(t_string!(i18n, add_project.error_platforms_required).to_string());
        }

        set_errors.set(e.clone());
        e.is_empty()
    };

    let toggle_tag = move |tag: Tag| {
        set_selected_tags.update(|tags| {
            if let Some(pos) = tags.iter().position(|t| *t == tag) {
                tags.remove(pos);
            } else {
                tags.push(tag);
            }
        });
        set_errors.update(|errs| errs.tags = None);
    };

    let toggle_platform = move |platform: Platform| {
        set_selected_platforms.update(|platforms| {
            if let Some(pos) = platforms.iter().position(|p| *p == platform) {
                platforms.remove(pos);
            } else {
                platforms.push(platform);
            }
        });
        set_errors.update(|errs| errs.platforms = None);
    };

    let on_submit = move |ev: leptos::web_sys::SubmitEvent| {
        ev.prevent_default();
        set_submit_error.set(None);

        if !validate() {
            return;
        }

        let payload = new_project_payload(
            title.get_untracked(),
            description.get_untracked(),
            extended_desc.get_untracked(),
            selected_tags.get_untracked(),
            selected_platforms.get_untracked(),
        );

        set_is_submitting.set(true);

        leptos::task::spawn_local(async move {
            let result = create_project(&payload).await;
            set_is_submitting.set(false);

            match result {
                Some(_) => {
                    // Refresh the project list from the backend so the new
                    // card appears with server-assigned author info.
                    let refreshed = crate::data::fetch_projects().await;
                    projects_ctx.set_projects.set(refreshed);
                    modal.close();
                    reset_form();
                }
                None => {
                    set_submit_error
                        .set(Some(t_string!(i18n, add_project.error_submit).to_string()));
                }
            }
        });
    };

    view! {
        <SearchModal
            open=modal.open
            on_close=on_close
        >
            <div class="space-y-4 flex flex-col min-h-0 max-h-[70vh]">
                <div class="flex items-center justify-between flex-shrink-0">
                    <h2 class="text-xl font-bold text-primary">{move || t_string!(i18n, add_project.title)}</h2>
                    <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50">
                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            {move || t_string!(i18n, search.keyboard_esc)}
                        </kbd>
                        <span>{move || t_string!(i18n, project_modal.hint_dismiss)}</span>
                    </div>
                </div>

                {move || {
                    if current_user.is_loading.get() {
                        return view! {
                            <div class="flex items-center justify-center py-8">
                                <span class="loading loading-spinner loading-md text-primary" aria-hidden="true"></span>
                            </div>
                        }.into_any();
                    }

                    if current_user.account.get().is_none() {
                        return view! {
                            <div class="space-y-4 flex flex-col">
                                <p class="text-sm text-base-content/80">
                                    {move || t_string!(i18n, add_project.login_required)}
                                </p>
                                <button
                                    type="button"
                                    class="btn btn-primary btn-lift"
                                    on:click=move |_| {
                                        modal.close();
                                        login_modal.open();
                                    }
                                >
                                    {move || t_string!(i18n, add_project.login_action)}
                                </button>
                            </div>
                        }.into_any();
                    }

                    view! {
                        <form class="space-y-4 flex flex-col min-h-0 overflow-hidden" on:submit=on_submit>
                            <div class="overflow-y-auto flex-1 min-h-0 space-y-4 pr-1">
                                <div>
                                    <label class="block text-sm font-medium text-base-content mb-1" for="add-project-title">
                                        {move || t_string!(i18n, add_project.field_title)}
                                    </label>
                                    <input
                                        node_ref=title_input_ref
                                        id="add-project-title"
                                        type="text"
                                        class="input w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none"
                                        on:input=move |ev| {
                                            set_title.set(event_target_value(&ev));
                                            set_errors.update(|errs| errs.title = None);
                                        }
                                        disabled=move || is_submitting.get()
                                    />
                                    {move || errors.get().title.map(|msg| view! {
                                        <p class="text-error text-xs mt-1">{msg}</p>
                                    })}
                                </div>

                                <div>
                                    <label class="block text-sm font-medium text-base-content mb-1" for="add-project-description">
                                        {move || t_string!(i18n, add_project.field_description)}
                                    </label>
                                    <textarea
                                        node_ref=desc_input_ref
                                        id="add-project-description"
                                        class="textarea w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none min-h-[4rem]"
                                        on:input=move |ev| {
                                            set_description.set(event_target_value(&ev));
                                            set_errors.update(|errs| errs.description = None);
                                        }
                                        disabled=move || is_submitting.get()
                                    ></textarea>
                                    {move || errors.get().description.map(|msg| view! {
                                        <p class="text-error text-xs mt-1">{msg}</p>
                                    })}
                                </div>

                                <div>
                                    <label class="block text-sm font-medium text-base-content mb-1" for="add-project-extended">
                                        {move || t_string!(i18n, add_project.field_extended)}
                                    </label>
                                    <textarea
                                        node_ref=extended_input_ref
                                        id="add-project-extended"
                                        class="textarea w-full rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none min-h-[6rem]"
                                        on:input=move |ev| {
                                            set_extended_desc.set(event_target_value(&ev));
                                        }
                                        disabled=move || is_submitting.get()
                                    ></textarea>
                                    <p class="text-xs text-base-content/50 mt-1">{move || t_string!(i18n, add_project.markdown_hint)}</p>
                                </div>

                                <div>
                                    <span class="block text-sm font-medium text-base-content mb-2">{move || t_string!(i18n, add_project.field_tags)}</span>
                                    <div class="flex flex-wrap gap-2" role="group" aria-label=move || t_string!(i18n, add_project.field_tags)>
                                        {Tag::all().into_iter().map(|tag| {
                                            let tag_clone = tag;
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        let selected = selected_tags.get().contains(&tag_clone);
                                                        format!(
                                                            "badge badge-sm badge-outline rounded-none cursor-pointer transition-colors {}",
                                                            if selected {
                                                                "bg-primary/20 border-primary text-primary"
                                                            } else {
                                                                "border-base-content/20 text-base-content/70 hover:border-primary/50"
                                                            }
                                                        )
                                                    }
                                                    on:click=move |_| toggle_tag(tag_clone)
                                                    disabled=move || is_submitting.get()
                                                    aria-pressed=move || selected_tags.get().contains(&tag_clone).to_string()
                                                >
                                                    {tag_clone.label()}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                    {move || errors.get().tags.map(|msg| view! {
                                        <p class="text-error text-xs mt-1">{msg}</p>
                                    })}
                                </div>

                                <div>
                                    <span class="block text-sm font-medium text-base-content mb-2">{move || t_string!(i18n, add_project.field_platforms)}</span>
                                    <div class="flex flex-wrap gap-2" role="group" aria-label=move || t_string!(i18n, add_project.field_platforms)>
                                        {Platform::all().into_iter().map(|platform| {
                                            let platform_clone = platform;
                                            view! {
                                                <button
                                                    type="button"
                                                    class=move || {
                                                        let selected = selected_platforms.get().contains(&platform_clone);
                                                        format!(
                                                            "badge badge-sm badge-outline rounded-none cursor-pointer transition-colors {}",
                                                            if selected {
                                                                "bg-primary/20 border-primary text-primary"
                                                            } else {
                                                                "border-base-content/20 text-base-content/70 hover:border-primary/50"
                                                            }
                                                        )
                                                    }
                                                    on:click=move |_| toggle_platform(platform_clone)
                                                    disabled=move || is_submitting.get()
                                                    aria-pressed=move || selected_platforms.get().contains(&platform_clone).to_string()
                                                >
                                                    {platform_clone.label()}
                                                </button>
                                            }
                                        }).collect_view()}
                                    </div>
                                    {move || errors.get().platforms.map(|msg| view! {
                                        <p class="text-error text-xs mt-1">{msg}</p>
                                    })}
                                </div>

                                {move || submit_error.get().map(|msg| view! {
                                    <p class="text-error text-sm">{msg}</p>
                                })}
                            </div>

                            <div class="flex justify-end gap-2 flex-shrink-0 pt-2 border-t border-base-content/10">
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-lift"
                                    on:click=move |_| modal.close()
                                    disabled=move || is_submitting.get()
                                >
                                    {move || t_string!(i18n, add_project.cancel)}
                                </button>
                                <button
                                    type="submit"
                                    class="btn btn-primary btn-lift"
                                    disabled=move || is_submitting.get()
                                >
                                    {move || {
                                        let label = if is_submitting.get() {
                                            t_string!(i18n, add_project.submitting)
                                        } else {
                                            t_string!(i18n, add_project.submit)
                                        };
                                        view! {
                                            <span class="flex items-center gap-2">
                                                {move || if is_submitting.get() {
                                                    Some(view! {
                                                        <span class="loading loading-spinner loading-xs" aria-hidden="true"></span>
                                                    })
                                                } else {
                                                    None
                                                }}
                                                <span>{label}</span>
                                            </span>
                                        }
                                    }}
                                </button>
                            </div>
                        </form>
                    }.into_any()
                }}
            </div>
        </SearchModal>
    }
}

/// Reads the value from an input/textarea event target.
fn event_target_value(ev: &leptos::web_sys::Event) -> String {
    ev.target()
        .and_then(|t| {
            t.dyn_into::<leptos::web_sys::HtmlTextAreaElement>()
                .ok()
                .map(|textarea| textarea.value())
        })
        .or_else(|| {
            ev.target()
                .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
                .map(|input| input.value())
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_errors_empty_by_default() {
        let errors = FormErrors::default();
        assert!(errors.is_empty());
    }

    #[test]
    fn form_errors_not_empty_when_any_field_set() {
        let errors = FormErrors {
            title: Some("required".to_string()),
            description: None,
            tags: None,
            platforms: None,
        };
        assert!(!errors.is_empty());
    }

    #[test]
    fn form_errors_empty_after_closing_all_errors() {
        let errors = FormErrors {
            title: None,
            description: None,
            tags: None,
            platforms: None,
        };
        assert!(errors.is_empty());
    }
}
