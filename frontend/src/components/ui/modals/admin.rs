use crate::components::ui::modals::search::SearchModal;
use crate::components::ui::toast::Toast;
use crate::contexts::{AdminModalContext, MetadataContext};
use crate::data::{
    create_platform, create_tag, delete_platform, delete_tag, update_platform, update_tag,
};
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;

fn tag_id(tag: &crate::metadata::tags::Tag) -> String {
    tag.id.clone()
}

fn tag_label(tag: &crate::metadata::tags::Tag) -> String {
    tag.label.clone()
}

fn tag_color(tag: &crate::metadata::tags::Tag) -> String {
    tag.color.clone()
}

fn platform_id(platform: &crate::metadata::platforms::Platform) -> String {
    platform.id.clone()
}

fn platform_label(platform: &crate::metadata::platforms::Platform) -> String {
    platform.label.clone()
}

fn platform_color(platform: &crate::metadata::platforms::Platform) -> String {
    platform.color.clone()
}

const MAX_LABEL_LENGTH: usize = 50;
const MAX_COLOR_LENGTH: usize = 100;
const MAX_ID_LENGTH: usize = 50;

/// Modal dialog that lets administrators manage tags and platforms.
#[component]
pub fn AdminModal() -> impl IntoView {
    let modal = AdminModalContext::use_context();
    let metadata = MetadataContext::use_context();

    let (toast_visible, set_toast_visible) = signal(false);
    let (toast_message, set_toast_message) = signal(String::new());
    let dismiss_toast = Callback::new(move |()| set_toast_visible.set(false));

    let show_toast = move |message: String| {
        set_toast_message.set(message);
        set_toast_visible.set(true);
        leptos::task::spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(2500).await;
            set_toast_visible.set(false);
        });
    };

    let refresh = move || metadata.refresh();

    let on_create_tag = Callback::new(move |(id, label, color): (String, String, String)| {
        let refresh = refresh;
        leptos::task::spawn_local(async move {
            match create_tag(id, label, color).await {
                Ok(_) => {
                    refresh();
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to create tag: {}", err.message()).into(),
                    );
                    show_toast(format!("Failed to create tag: {}", err.message()));
                }
            }
        });
    });

    let on_update_tag = Callback::new(move |(id, label, color): (String, String, String)| {
        let refresh = refresh;
        leptos::task::spawn_local(async move {
            match update_tag(&id, label, color).await {
                Ok(_) => {
                    refresh();
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to update tag: {}", err.message()).into(),
                    );
                    show_toast(format!("Failed to update tag: {}", err.message()));
                }
            }
        });
    });

    let on_delete_tag = Callback::new(move |id: String| {
        let refresh = refresh;
        leptos::task::spawn_local(async move {
            match delete_tag(&id).await {
                Ok(()) => {
                    refresh();
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to delete tag: {}", err.message()).into(),
                    );
                    show_toast(format!("Failed to delete tag: {}", err.message()));
                }
            }
        });
    });

    let on_create_platform = Callback::new(move |(id, label, color): (String, String, String)| {
        let refresh = refresh;
        leptos::task::spawn_local(async move {
            match create_platform(id, label, color).await {
                Ok(_) => {
                    refresh();
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to create platform: {}", err.message()).into(),
                    );
                    show_toast(format!("Failed to create platform: {}", err.message()));
                }
            }
        });
    });

    let on_update_platform = Callback::new(move |(id, label, color): (String, String, String)| {
        let refresh = refresh;
        leptos::task::spawn_local(async move {
            match update_platform(&id, label, color).await {
                Ok(_) => {
                    refresh();
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to update platform: {}", err.message()).into(),
                    );
                    show_toast(format!("Failed to update platform: {}", err.message()));
                }
            }
        });
    });

    let on_delete_platform = Callback::new(move |id: String| {
        let refresh = refresh;
        leptos::task::spawn_local(async move {
            match delete_platform(&id).await {
                Ok(()) => {
                    refresh();
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to delete platform: {}", err.message()).into(),
                    );
                    show_toast(format!("Failed to delete platform: {}", err.message()));
                }
            }
        });
    });

    let on_close = Callback::new(move |()| modal.close());

    view! {
        <SearchModal
            open=modal.open
            on_close=on_close
            container_class=Signal::derive(|| "w-full max-w-4xl max-h-[85vh] flex flex-col".to_string())
        >
            <div class="flex flex-col h-full min-h-0 space-y-4">
                <Toast
                    message=Signal::derive(move || toast_message.get())
                    visible=Signal::derive(move || toast_visible.get())
                    on_dismiss=dismiss_toast
                />
                <h2 class="text-xl font-bold text-primary">"Admin"</h2>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6 min-h-0 overflow-y-auto">
                    <MetadataEditor
                        title="Tags"
                        items=Signal::derive({
                            let metadata = metadata;
                            move || metadata.tags.get()
                        })
                        id_fn=tag_id
                        label_fn=tag_label
                        color_fn=tag_color
                        on_create=on_create_tag
                        on_update=on_update_tag
                        on_delete=on_delete_tag
                    />
                    <MetadataEditor
                        title="Platforms"
                        items=Signal::derive({
                            let metadata = metadata;
                            move || metadata.platforms.get()
                        })
                        id_fn=platform_id
                        label_fn=platform_label
                        color_fn=platform_color
                        on_create=on_create_platform
                        on_update=on_update_platform
                        on_delete=on_delete_platform
                    />
                </div>
            </div>
        </SearchModal>
    }
}

/// Editable list of metadata items (tags or platforms) with create, rename, and
/// delete controls.
#[component]
fn MetadataEditor<T: Clone + Send + Sync + 'static>(
    #[prop(into)] title: &'static str,
    #[prop(into)] items: Signal<Vec<T>>,
    id_fn: fn(&T) -> String,
    label_fn: fn(&T) -> String,
    color_fn: fn(&T) -> String,
    #[prop(into)] on_create: Callback<(String, String, String)>,
    #[prop(into)] on_update: Callback<(String, String, String)>,
    #[prop(into)] on_delete: Callback<String>,
) -> impl IntoView {
    let (editing_id, set_editing_id) = signal::<Option<String>>(None);
    let (draft_label, set_draft_label) = signal(String::new());
    let (draft_color, set_draft_color) = signal(String::new());
    let (new_id, set_new_id) = signal(String::new());
    let (new_label, set_new_label) = signal(String::new());
    let (new_color, set_new_color) = signal(String::new());

    let start_edit = Callback::new(move |(id, label, color): (String, String, String)| {
        set_editing_id.set(Some(id));
        set_draft_label.set(label);
        set_draft_color.set(color);
    });

    let cancel_edit = Callback::new(move |()| {
        set_editing_id.set(None);
        set_draft_label.set(String::new());
        set_draft_color.set(String::new());
    });

    let commit_edit = Callback::new(move |id: String| {
        let label = draft_label.get_untracked();
        let color = draft_color.get_untracked();
        if label.trim().is_empty() || color.trim().is_empty() {
            return;
        }
        on_update.run((id, label, color));
        cancel_edit.run(());
    });

    let submit_create = Callback::new(move |()| {
        let id = new_id.get_untracked();
        let label = new_label.get_untracked();
        let color = new_color.get_untracked();
        if id.trim().is_empty() || label.trim().is_empty() || color.trim().is_empty() {
            return;
        }
        on_create.run((id, label, color));
        set_new_id.set(String::new());
        set_new_label.set(String::new());
        set_new_color.set(String::new());
    });

    view! {
        <div class="flex flex-col min-h-0 space-y-3">
            <h3 class="text-sm font-semibold text-base-content">{title}</h3>
            <div class="space-y-2 overflow-y-auto pr-1">
                {move || {
                    let items = items.get();
                    if items.is_empty() {
                        return view! {
                            <p class="text-base-content/50 text-sm">"No items yet."</p>
                        }
                            .into_any();
                    }
                    view! {
                        <ul class="space-y-2" role="list">
                            {items
                                .into_iter()
                                .map(|item| {
                                    let id = id_fn(&item);
                                    let label = label_fn(&item);
                                    let color = color_fn(&item);
                                    view! {
                                        <li class="flex items-center gap-2 group" role="listitem">
                                            {move || {
                                                let id = id.clone();
                                                let label = label.clone();
                                                let color = color.clone();
                                                let editing = editing_id.get().as_ref() == Some(&id);
                                                if editing {
                                                    view! {
                                                        <div class="flex-1 flex flex-col gap-1">
                                                            <input
                                                                type="text"
                                                                class="input input-xs input-bordered w-full"
                                                                maxlength=MAX_LABEL_LENGTH.to_string()
                                                                prop:value=move || draft_label.get()
                                                                on:input=move |ev| set_draft_label.set(event_target_value(&ev))
                                                                on:keyup={
                                                                    let id = id.clone();
                                                                    move |ev| {
                                                                        if ev.key().as_str() == "Enter" {
                                                                            commit_edit.run(id.clone());
                                                                        } else if ev.key().as_str() == "Escape" {
                                                                            cancel_edit.run(());
                                                                        }
                                                                    }
                                                                }
                                                                placeholder="Label"
                                                            />
                                                            <input
                                                                type="text"
                                                                class="input input-xs input-bordered w-full"
                                                                maxlength=MAX_COLOR_LENGTH.to_string()
                                                                prop:value=move || draft_color.get()
                                                                on:input=move |ev| set_draft_color.set(event_target_value(&ev))
                                                                on:keyup={
                                                                    let id = id.clone();
                                                                    move |ev| {
                                                                        if ev.key().as_str() == "Enter" {
                                                                            commit_edit.run(id.clone());
                                                                        } else if ev.key().as_str() == "Escape" {
                                                                            cancel_edit.run(());
                                                                        }
                                                                    }
                                                                }
                                                                placeholder="background-color:#1d4ed8;color:#ffffff"
                                                            />
                                                        </div>
                                                        <div class="flex items-center gap-1 flex-shrink-0">
                                                            <button
                                                                type="button"
                                                                class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-success hover:text-success"
                                                                aria-label="Save"
                                                                on:click={
                                                                    let id = id.clone();
                                                                    move |_| commit_edit.run(id.clone())
                                                                }
                                                            >
                                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                                                    <polyline points="20 6 9 17 4 12" />
                                                                </svg>
                                                            </button>
                                                            <button
                                                                type="button"
                                                                class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-base-content"
                                                                aria-label="Cancel"
                                                                on:click=move |_| cancel_edit.run(())
                                                            >
                                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                                                    <line x1="18" y1="6" x2="6" y2="18" />
                                                                    <line x1="6" y1="6" x2="18" y2="18" />
                                                                </svg>
                                                            </button>
                                                        </div>
                                                    }
                                                        .into_any()
                                                } else {
                                                    view! {
                                                        <span
                                                            class="badge badge-sm badge-outline rounded-none flex-shrink-0"
                                                            style=color.clone()
                                                        >
                                                            {label.clone()}
                                                        </span>
                                                        <span class="text-xs text-base-content/50 truncate flex-1">{id.clone()}</span>
                                                        <div class="flex items-center gap-1 flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
                                                            <button
                                                                type="button"
                                                                class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-base-content/50 hover:text-primary"
                                                                aria-label="Edit"
                                                                on:click={
                                                                    let id = id.clone();
                                                                    let label = label.clone();
                                                                    let color = color.clone();
                                                                    move |_| {
                                                                        start_edit.run((id.clone(), label.clone(), color.clone()));
                                                                    }
                                                                }
                                                            >
                                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                                                    <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                                                                </svg>
                                                            </button>
                                                            <button
                                                                type="button"
                                                                class="btn btn-ghost btn-xs p-1 h-auto min-h-0 text-error hover:text-error"
                                                                aria-label="Delete"
                                                                on:click={
                                                                    let id = id.clone();
                                                                    move |_| on_delete.run(id.clone())
                                                                }
                                                            >
                                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
                                                                    <path d="M3 6h18" />
                                                                    <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
                                                                    <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
                                                                </svg>
                                                            </button>
                                                        </div>
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }
                        .into_any()
                }}
            </div>
            <div class="border-t border-base-content/10 pt-3 space-y-2">
                <p class="text-xs font-semibold text-base-content/70">"Create new"</p>
                <input
                    type="text"
                    class="input input-xs input-bordered w-full"
                    maxlength=MAX_ID_LENGTH.to_string()
                    prop:value=move || new_id.get()
                    on:input=move |ev| set_new_id.set(event_target_value(&ev))
                    placeholder="id"
                />
                <input
                    type="text"
                    class="input input-xs input-bordered w-full"
                    maxlength=MAX_LABEL_LENGTH.to_string()
                    prop:value=move || new_label.get()
                    on:input=move |ev| set_new_label.set(event_target_value(&ev))
                    placeholder="Label"
                />
                <input
                    type="text"
                    class="input input-xs input-bordered w-full"
                    maxlength=MAX_COLOR_LENGTH.to_string()
                    prop:value=move || new_color.get()
                    on:input=move |ev| set_new_color.set(event_target_value(&ev))
                    on:keyup=move |ev| {
                        if ev.key().as_str() == "Enter" {
                            submit_create.run(());
                        }
                    }
                    placeholder="background-color:#1d4ed8;color:#ffffff"
                />
                <button
                    type="button"
                    class="btn btn-primary btn-xs w-full"
                    on:click=move |_| submit_create.run(())
                >
                    "Create"
                </button>
            </div>
        </div>
    }
}

fn event_target_value(ev: &leptos::web_sys::Event) -> String {
    ev.target()
        .and_then(|t| t.dyn_into::<leptos::web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}
