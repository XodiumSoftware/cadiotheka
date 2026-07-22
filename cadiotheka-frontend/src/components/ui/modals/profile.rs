use crate::components::ui::corner_frame::CornerFrame;
use crate::components::ui::modals::search::SearchModal;
use crate::components::ui::toast::Toast;
use crate::contexts::{CurrentUserContext, ProfileModalContext};
use crate::utils::{
    encode_redirect_url, format_time_full, login_url, placeholder_color, placeholder_letter,
};
use gloo_net::http::Request;
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

    let (linked_providers, set_linked_providers) = signal::<Vec<String>>(Vec::new());

    Effect::new(move |_| {
        if modal.open.get() && is_editable {
            leptos::task::spawn_local(async move {
                let providers = crate::contexts::current_user::fetch_linked_providers().await;
                set_linked_providers.set(providers);
            });
        }
    });

    let start_link_oauth = move |provider: &'static str| {
        leptos::task::spawn_local(async move {
            let url = encode_redirect_url(&login_url(provider));
            let Ok(resp) = Request::get(&url)
                .credentials(web_sys::RequestCredentials::Include)
                .send()
                .await
            else {
                return;
            };
            let Ok(parsed) = resp.json::<AuthUrlResponse>().await else {
                return;
            };
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&parsed.url);
            }
        });
    };

    let start_unlink = move |provider: &'static str| {
        leptos::task::spawn_local(async move {
            if crate::contexts::current_user::unlink_provider(provider).await {
                set_linked_providers.update(|providers| {
                    providers.retain(|p| p != provider);
                });
            }
        });
    };

    let is_connected = move |provider: &str| {
        linked_providers.get().contains(&provider.to_string()) || account.provider == provider
    };

    let is_github_connected = Memo::new({
        let is_connected = is_connected.clone();
        move |_| is_connected("github")
    });
    let is_google_connected = Memo::new(move |_| is_connected("google"));

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

                {if is_editable {
                    Some(view! {
                        <div class="flex items-stretch gap-2">
                            <div class="flex-shrink-0 flex bg-surface-light p-1 relative w-8">
                                <CornerFrame
                                    style="square"
                                    black=true
                                    class="h-full w-full flex items-center justify-center"
                                >
                                    <h2 class="text-lg font-bold tracking-tight text-transparent bg-base-100 bg-clip-text flex flex-col items-center justify-center leading-none">
                                        <span>"L"</span>
                                        <span>"I"</span>
                                        <span>"N"</span>
                                        <span>"K"</span>
                                    </h2>
                                </CornerFrame>
                            </div>
                            <div class="flex items-stretch gap-2 flex-1">
                                <button
                                    type="button"
                                    class=move || {
                                        let connected = is_github_connected.get();
                                        let base = "relative btn btn-sm flex-1 flex items-center justify-center gap-2 rounded-none border border-base-content/20 h-auto min-h-0 py-2 btn-outline hover:btn-primary";
                                        if connected {
                                            format!("{} btn-ghost", base)
                                        } else {
                                            base.to_string()
                                        }
                                    }
                                    on:click=move |_| {
                                        if is_github_connected.get() {
                                            start_unlink("github");
                                        } else {
                                            start_link_oauth("github");
                                        }
                                    }
                                >
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                                        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12Z"/>
                                    </svg>
                                    {move || if is_github_connected.get() { "GitHub connected" } else { "Connect GitHub" }}
                                    {move || if is_github_connected.get() {
                                        view! {
                                            <span class="absolute -top-2 -right-2 flex h-6 w-6 items-center justify-center rounded-full bg-error text-error-content border-2 border-base-100 shadow-sm" title="Disconnect GitHub">
                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M5 12h14"/>
                                                </svg>
                                            </span>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }}
                                </button>
                                <button
                                    type="button"
                                    class=move || {
                                        let connected = is_google_connected.get();
                                        let base = "relative btn btn-sm flex-1 flex items-center justify-center gap-2 rounded-none border border-base-content/20 h-auto min-h-0 py-2 btn-outline hover:btn-primary";
                                        if connected {
                                            format!("{} btn-ghost", base)
                                        } else {
                                            base.to_string()
                                        }
                                    }
                                    on:click=move |_| {
                                        if is_google_connected.get() {
                                            start_unlink("google");
                                        } else {
                                            start_link_oauth("google");
                                        }
                                    }
                                >
                                    <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                        <circle cx="12" cy="12" r="10"/>
                                        <text x="12" y="16" text-anchor="middle" font-size="12" fill="currentColor" stroke="none">"G"</text>
                                    </svg>
                                    {move || if is_google_connected.get() { "Google connected" } else { "Connect Google" }}
                                    {move || if is_google_connected.get() {
                                        view! {
                                            <span class="absolute -top-2 -right-2 flex h-6 w-6 items-center justify-center rounded-full bg-error text-error-content border-2 border-base-100 shadow-sm" title="Disconnect Google">
                                                <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round">
                                                    <path d="M5 12h14"/>
                                                </svg>
                                            </span>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }}
                                </button>
                            </div>
                        </div>
                    })
                } else {
                    None
                }}
            </div>
        </div>
    }
}

#[derive(Debug, serde::Deserialize)]
struct AuthUrlResponse {
    url: String,
}
