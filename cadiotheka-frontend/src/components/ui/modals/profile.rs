use crate::components::ui::modals::search::SearchModal;
use crate::contexts::{CurrentUserContext, ProfileModalContext};
use crate::utils::{format_time_full, placeholder_color, placeholder_letter};
use leptos::prelude::*;

/// Maximum length for a user-written bio, matching GitHub's profile bio limit.
const MAX_BIO_LENGTH: usize = 160;

/// Modal dialog that displays profile information for a selected account.
#[component]
pub fn ProfileModal() -> impl IntoView {
    let modal = ProfileModalContext::use_context();
    let on_close = move |_| modal.close();

    view! {
        <SearchModal
            open=modal.open
            on_close=on_close
        >
            {move || {
                modal.account.get().map(|account| {
                    view! {
                        <ProfileModalContent account=account />
                    }
                })
            }}
        </SearchModal>
    }
}

#[component]
fn ProfileModalContent(#[prop(into)] account: crate::data::AccountData) -> impl IntoView {
    let current_user = CurrentUserContext::use_context();
    let modal = ProfileModalContext::use_context();
    let is_editable = current_user
        .account
        .get()
        .is_some_and(|me| me.id == account.id);

    let (editing, set_editing) = signal(false);
    let (draft, set_draft) = signal(account.bio.clone());
    let (bio, set_bio) = signal(account.bio.clone());

    let start_edit = move |_| {
        set_draft.set(bio.get());
        set_editing.set(true);
    };

    let cancel_edit = move || {
        set_editing.set(false);
    };

    let commit_edit = move |draft_value: String| {
        let current_user = current_user;
        let modal_account = modal.set_account;
        let set_current_user = current_user.set_account;
        let set_bio = set_bio;
        let set_editing = set_editing;

        leptos::task::spawn_local(async move {
            if let Some(new_bio) = crate::contexts::current_user::update_bio(draft_value).await {
                set_bio.set(new_bio.clone());
                modal_account.update(|opt| {
                    if let Some(acc) = opt.as_mut() {
                        acc.bio.clone_from(&new_bio);
                    }
                });
                set_current_user.update(|opt| {
                    if let Some(acc) = opt.as_mut() {
                        acc.bio.clone_from(&new_bio);
                    }
                });
            }
            set_editing.set(false);
        });
    };

    let letter = placeholder_letter(&account.username);
    let bg = placeholder_color(&account.username);
    let display_name = account.display_name.clone();
    let username = account.username.clone();
    let avatar_alt = format!("{}'s avatar", display_name);
    let role_label = move || match account.role {
        crate::data::AccountRole::Creator => "Creator".to_string(),
        crate::data::AccountRole::Admin => "Admin".to_string(),
    };

    view! {
        <div class="space-y-4 flex flex-col min-h-0">
            <div class="flex items-start gap-4">
                {match account.avatar_url.clone() {
                    Some(url) => view! {
                        <img
                            class="flex-shrink-0 w-16 h-16 rounded object-cover"
                            src=url
                            alt=avatar_alt.clone()
                            aria-hidden="true"
                        />
                    }
                    .into_any(),
                    None => view! {
                        <div class=format!("flex-shrink-0 w-16 h-16 rounded flex items-center justify-center text-white font-bold text-xl {}", bg)
                            aria-hidden="true"
                        >
                            {letter.clone()}
                        </div>
                    }
                    .into_any(),
                }}
                <div class="min-w-0 flex-1 flex flex-col gap-1">
                    <div class="flex items-center gap-2">
                        <h2 class="text-xl font-bold text-primary leading-tight truncate">
                            {display_name.clone()}
                        </h2>
                        <span class="badge badge-xs badge-outline rounded-none border-base-content/20 text-base-content/70 self-center">
                            {role_label}
                        </span>
                    </div>
                    <p class="text-base-content/70 text-sm">
                        {"@"}
                        {username.clone()}
                    </p>
                </div>
                <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50 flex-shrink-0">
                    <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">esc</kbd>
                    <span>to close</span>
                </div>
            </div>
            <hr class="border-base-content/10" />
            <div class="space-y-2 text-sm text-base-content/80">
                <p>
                    <span class="font-semibold text-base-content">Email:</span>
                    <span class="ml-1">{account.email.clone()}</span>
                </p>
                <p>
                    <span class="font-semibold text-base-content">Joined:</span>
                    <span class="ml-1">{format_time_full(account.created_at)}</span>
                </p>
                <div class="flex items-start gap-2">
                    <span class="font-semibold text-base-content flex-shrink-0">Bio:</span>
                    {move || {
                        if editing.get() {
                            view! {
                                <div class="flex items-center gap-2 flex-1">
                                    <input
                                        class="input input-sm input-bordered flex-1 text-base-content"
                                        type="text"
                                        maxlength=MAX_BIO_LENGTH.to_string()
                                        prop:value=draft.get()
                                        on:input=move |ev| set_draft.set(event_target_value(&ev))
                                        on:keyup=move |ev| {
                                            match ev.key().as_str() {
                                                "Enter" => commit_edit(draft.get()),
                                                "Escape" => cancel_edit(),
                                                _ => {}
                                            }
                                        }
                                        autofocus
                                    />
                                    <span class="text-xs text-base-content/50 flex-shrink-0">
                                        {move || format!("{}/{}", draft.get().len(), MAX_BIO_LENGTH)}
                                    </span>
                                </div>
                            }
                            .into_any()
                        } else {
                            view! {
                                <div class="flex items-center gap-2 flex-1">
                                    <span class="text-base-content/70">{bio.get()}</span>
                                    {is_editable.then(|| view! {
                                        <button
                                            type="button"
                                            class="btn btn-ghost btn-xs p-1 h-auto min-h-0"
                                            aria-label="Edit bio"
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
                </div>
            </div>
        </div>
    }
}
