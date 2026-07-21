use crate::components::ui::corner_frame::CornerFrame;
use crate::components::ui::modals::search::SearchModal;
use crate::components::ui::toast::Toast;
use crate::contexts::{CurrentUserContext, ProfileModalContext};
use crate::utils::{format_time_full, placeholder_color, placeholder_letter};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use web_sys::window;
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

    let (toast_visible, set_toast_visible) = signal(false);
    let show_toast = move || {
        set_toast_visible.set(true);
        leptos::task::spawn_local(async move {
            TimeoutFuture::new(1500).await;
            set_toast_visible.set(false);
        });
    };

    let copy_username = {
        let username = username.clone();
        move |_| {
            let username = username.clone();
            leptos::task::spawn_local(async move {
                if let Some(clipboard) = window().map(|w| w.navigator().clipboard()) {
                    let _ = clipboard.write_text(&username).await;
                }
                show_toast();
            });
        }
    };

    let dismiss_toast = Callback::new(move |_| set_toast_visible.set(false));

    view! {
        <Toast
            message=Signal::derive(move || "Copied username to clipboard".to_string())
            visible=Signal::derive(move || toast_visible.get())
            on_dismiss=dismiss_toast
        />
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
                        <button
                            type="button"
                            class="text-left group cursor-pointer tooltip tooltip-bottom"
                            data-tip={format!("@{}", username)}
                            aria-label={format!("Copy username @{}", username)}
                            on:click=copy_username
                        >
                            <h2 class="text-xl font-bold text-primary leading-tight truncate group-hover:text-primary/80 transition-colors">
                                {display_name.clone()}
                            </h2>
                        </button>
                        <span class="badge badge-xs badge-outline rounded-none border-base-content/20 text-base-content/70 self-center">
                            {role_label}
                        </span>
                    </div>
                    <div class="flex flex-col gap-0.5 text-xs text-base-content/60">
                        <span class="inline-flex items-center gap-1">
                            <svg class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <rect x="2" y="4" width="20" height="16" rx="2" />
                                <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
                            </svg>
                            <span class="leading-none">{account.email.clone()}</span>
                        </span>
                        <span class="inline-flex items-center gap-1">
                            <svg class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <circle cx="12" cy="12" r="10" />
                                <path d="M12 6v6l4 2" />
                            </svg>
                            <span class="leading-none">{format_time_full(account.created_at)}</span>
                        </span>
                    </div>
                </div>
                <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50 flex-shrink-0">
                    <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">esc</kbd>
                    <span>to close</span>
                </div>
            </div>
            <hr class="border-base-content/10" />
            <div class="space-y-2 text-sm text-base-content/80">
                <div class="flex items-stretch gap-2">
                    <div class="flex-shrink-0 flex bg-surface-light p-1 relative w-8">
                        <CornerFrame
                            style="square"
                            black=true
                            class="h-full w-full flex items-center justify-center"
                        >
                            <h2 class="text-lg font-bold tracking-tight text-transparent bg-base-100 bg-clip-text flex flex-col items-center justify-center leading-none">
                                <span>"B"</span>
                                <span>"I"</span>
                                <span>"O"</span>
                            </h2>
                        </CornerFrame>
                    </div>
                    {move || {
                        if editing.get() {
                            view! {
                                <div class="space-y-2 flex-1">
                                    <textarea
                                        class=move || {
                                            let at_max = draft.get().len() >= MAX_BIO_LENGTH;
                                            format!(
                                                "textarea w-full min-h-[5rem] rounded-none bg-transparent border-base-content/20 focus:border-primary focus:outline-none {}",
                                                if at_max { "hover:border-error" } else { "" }
                                            )
                                        }
                                        maxlength=MAX_BIO_LENGTH.to_string()
                                        prop:value=draft.get()
                                        on:input=move |ev| set_draft.set(event_target_value(&ev))
                                        on:keyup=move |ev| {
                                            if ev.key().as_str() == "Escape" {
                                                cancel_edit();
                                            }
                                        }
                                        autofocus
                                    ></textarea>
                                    <div class="flex items-center justify-between">
                                        <span class=move || {
                                            if draft.get().len() >= MAX_BIO_LENGTH {
                                                "text-xs text-error"
                                            } else {
                                                "text-xs text-base-content/50"
                                            }
                                        }>
                                            {move || format!("{}/{}", draft.get().len(), MAX_BIO_LENGTH)}
                                        </span>
                                        <div class="flex gap-2">
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-xs"
                                                on:click=move |_| cancel_edit()
                                            >"Cancel"</button>
                                            <button
                                                type="button"
                                                class="btn btn-primary btn-xs"
                                                on:click=move |_| commit_edit(draft.get())
                                            >"Save"</button>
                                        </div>
                                    </div>
                                </div>
                            }
                            .into_any()
                        } else {
                            view! {
                                <div class="flex items-stretch gap-2 flex-1">
                                    {if is_editable {
                                        view! {
                                            <button
                                                type="button"
                                                class="group text-left relative border border-base-content/20 rounded-none p-2 flex-1 hover:border-primary transition-colors cursor-pointer"
                                                aria-label="Edit bio"
                                                on:click=start_edit
                                            >
                                                <span class="text-base-content/70 group-hover:text-base-content transition-colors">{bio.get()}</span>
                                                <div class="absolute inset-0 flex items-center justify-center bg-base-100/80 opacity-0 group-hover:opacity-100 transition-opacity">
                                                    <svg
                                                        class="w-5 h-5 text-primary"
                                                        viewBox="0 0 24 24"
                                                        fill="none"
                                                        stroke="currentColor"
                                                        stroke-width="2"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                    >
                                                        <path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
                                                    </svg>
                                                </div>
                                            </button>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <span class="text-base-content/70 border border-base-content/20 rounded-none p-2 flex-1">{bio.get()}</span>
                                        }
                                            .into_any()
                                    }}
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
