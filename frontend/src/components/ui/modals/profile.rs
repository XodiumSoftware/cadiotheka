use crate::components::ui::corner_frame::CornerFrame;
use crate::components::ui::modals::base::BaseModal;
use crate::contexts::{CurrentUserContext, ProfileModalContext, ToastContext};
use crate::utils::{
    PasskeyInfo, delete_passkey, encode_redirect_url, fetch_passkeys, format_time_full,
    github_login_url, google_login_url, passkey_supported, placeholder_color, placeholder_letter,
    register_passkey,
};
use gloo_net::http::Request;
use leptos::prelude::*;
use web_sys::window;
const MAX_BIO_LENGTH: usize = 160;

/// Modal dialog that displays profile information for a selected account.
#[component]
pub fn ProfileModal() -> impl IntoView {
    let modal = ProfileModalContext::use_context();
    let on_close = move |()| modal.close();

    view! {
        <BaseModal
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
        </BaseModal>
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
            match crate::contexts::current_user::update_bio(draft_value).await {
                Ok(new_bio) => {
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
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to update bio: {}", err.message()).into(),
                    );
                }
            }
            set_editing.set(false);
        });
    };

    let letter = placeholder_letter(&account.username);
    let bg = placeholder_color(&account.username);
    let display_name = account.display_name.clone();
    let username = account.username.clone();
    let email = account.email.clone();
    let avatar_alt = format!("{display_name}'s avatar");
    let role_label = move || account.role.to_string();

    let copy_username = {
        let username = username.clone();
        let toast = ToastContext::use_context();
        move |_| {
            let username = username.clone();
            let toast = toast;
            leptos::task::spawn_local(async move {
                if let Some(clipboard) = window().map(|w| w.navigator().clipboard()) {
                    let _ = clipboard.write_text(&username).await;
                }
                toast.show("Copied username to clipboard");
            });
        }
    };

    let (linked_providers, set_linked_providers) = signal::<Vec<String>>(Vec::new());
    let (passkeys, set_passkeys) = signal::<Vec<PasskeyInfo>>(Vec::new());
    let (passkey_in_flight, set_passkey_in_flight) = signal(false);
    let (naming_passkey, set_naming_passkey) = signal(false);
    let (passkey_name, set_passkey_name) = signal(String::from("This device"));
    let can_passkey = passkey_supported();

    Effect::new(move |_| {
        if modal.open.get() && is_editable {
            leptos::task::spawn_local(async move {
                match crate::contexts::current_user::fetch_linked_providers().await {
                    Ok(providers) => set_linked_providers.set(providers),
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to fetch linked providers: {}", err.message()).into(),
                        );
                    }
                }
                match crate::utils::fetch_passkeys().await {
                    Ok(items) => set_passkeys.set(items),
                    Err(err) => {
                        leptos::web_sys::console::error_1(
                            &format!("Failed to fetch passkeys: {}", err.message()).into(),
                        );
                    }
                }
            });
        }
    });

    let start_link_oauth = move |provider: &'static str| {
        let toast = ToastContext::use_context();
        leptos::task::spawn_local(async move {
            let url = match provider {
                "github" => encode_redirect_url(&github_login_url()),
                "google" => encode_redirect_url(&google_login_url()),
                _ => return,
            };
            match fetch_link_url(&url).await {
                Ok(target) => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href(&target);
                    }
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to start provider linking: {err}").into(),
                    );
                    toast.show("Couldn't start provider linking. Please try again.");
                }
            }
        });
    };

    let start_unlink = move |provider: &'static str| {
        leptos::task::spawn_local(async move {
            match crate::contexts::current_user::unlink_provider(provider).await {
                Ok(()) => {
                    set_linked_providers.update(|providers| {
                        providers.retain(|p| p != provider);
                    });
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to unlink provider: {}", err.message()).into(),
                    );
                }
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

    let start_naming_passkey = move |_| {
        set_passkey_name.set(String::from("This device"));
        set_naming_passkey.set(true);
    };

    let cancel_naming_passkey = move |_| {
        set_naming_passkey.set(false);
    };

    let start_register_passkey = move |()| {
        let toast = ToastContext::use_context();
        let name = passkey_name.get().trim().to_string();
        set_passkey_in_flight.set(true);
        leptos::task::spawn_local(async move {
            match register_passkey(&name).await {
                Ok(()) => {
                    if let Ok(items) = fetch_passkeys().await {
                        set_passkeys.set(items);
                    }
                    toast.show("Passkey added");
                }
                Err(err) if err == "Registration cancelled" => {
                    toast.show("Passkey registration cancelled");
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to register passkey: {err}").into(),
                    );
                    toast.show("Couldn't add passkey. Please try again.");
                }
            }
            set_passkey_in_flight.set(false);
            set_naming_passkey.set(false);
        });
    };

    let delete_passkey_at = move |id: String| {
        let toast = ToastContext::use_context();
        leptos::task::spawn_local(async move {
            match delete_passkey(&id).await {
                Ok(()) => {
                    set_passkeys.update(|items| items.retain(|p| p.id != id));
                    toast.show("Passkey removed");
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to delete passkey: {}", err.message()).into(),
                    );
                    toast.show("Couldn't remove passkey. Please try again.");
                }
            }
        });
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
                            loading="lazy"
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
                            data-tip={format!("@{username}")}
                            aria-label={format!("Copy username @{username}")}
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
                        {(!email.is_empty()).then(|| {
                            view! {
                                <span class="inline-flex items-center gap-1">
                                    <svg class="w-3.5 h-3.5 flex-shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                        <rect x="2" y="4" width="20" height="16" rx="2" />
                                        <path d="m22 7-8.97 5.7a1.94 1.94 0 0 1-2.06 0L2 7" />
                                    </svg>
                                    <span class="leading-none">{email.clone()}</span>
                                </span>
                            }
                        })}
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
                                        let base = "relative btn btn-sm flex-1 flex items-center justify-center gap-2 rounded-none border border-base-content/20 h-auto min-h-0 py-2 btn-outline hover:btn-primary overflow-hidden group";
                                        if connected {
                                            format!("{base} btn-ghost")
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
                                    <span class="flex items-center justify-center gap-2 transition-opacity group-hover:opacity-0">
                                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                                            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12Z"/>
                                        </svg>
                                        {move || if is_github_connected.get() { "GitHub connected" } else { "Connect GitHub" }}
                                    </span>
                                    {move || if is_github_connected.get() {
                                        view! {
                                            <span class="absolute inset-0 flex items-center justify-center gap-2 bg-error/80 text-error-content opacity-0 group-hover:opacity-100 transition-opacity" aria-label="Disconnect GitHub">
                                                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <line x1="5" y1="12" x2="19" y2="12" />
                                                </svg>
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="absolute inset-0 flex items-center justify-center gap-2 bg-success/80 text-success-content opacity-0 group-hover:opacity-100 transition-opacity" aria-label="Connect GitHub">
                                                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <line x1="12" y1="5" x2="12" y2="19" />
                                                    <line x1="5" y1="12" x2="19" y2="12" />
                                                </svg>
                                            </span>
                                        }.into_any()
                                    }}
                                </button>
                                <button
                                    type="button"
                                    class=move || {
                                        let connected = is_google_connected.get();
                                        let base = "relative btn btn-sm flex-1 flex items-center justify-center gap-2 rounded-none border border-base-content/20 h-auto min-h-0 py-2 btn-outline hover:btn-primary overflow-hidden group";
                                        if connected {
                                            format!("{base} btn-ghost")
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
                                    <span class="flex items-center justify-center gap-2 transition-opacity group-hover:opacity-0">
                                        <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true">
                                            <circle cx="12" cy="12" r="10"/>
                                            <text x="12" y="16" text-anchor="middle" font-size="12" fill="currentColor" stroke="none">"G"</text>
                                        </svg>
                                        {move || if is_google_connected.get() { "Google connected" } else { "Connect Google" }}
                                    </span>
                                    {move || if is_google_connected.get() {
                                        view! {
                                            <span class="absolute inset-0 flex items-center justify-center gap-2 bg-error/80 text-error-content opacity-0 group-hover:opacity-100 transition-opacity" aria-label="Disconnect Google">
                                                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <line x1="5" y1="12" x2="19" y2="12" />
                                                </svg>
                                            </span>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <span class="absolute inset-0 flex items-center justify-center gap-2 bg-success/80 text-success-content opacity-0 group-hover:opacity-100 transition-opacity" aria-label="Connect Google">
                                                <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                    <line x1="12" y1="5" x2="12" y2="19" />
                                                    <line x1="5" y1="12" x2="19" y2="12" />
                                                </svg>
                                            </span>
                                        }.into_any()
                                    }}
                                </button>
                            </div>
                        </div>

                        <div class="flex items-stretch gap-2">
                            <div class="flex-shrink-0 flex bg-surface-light p-1 relative w-8">
                                <CornerFrame
                                    style="square"
                                    black=true
                                    class="h-full w-full flex items-center justify-center"
                                >
                                    <h2 class="text-lg font-bold tracking-tight text-transparent bg-base-100 bg-clip-text flex flex-col items-center justify-center leading-none">
                                        <span>"P"</span>
                                        <span>"A"</span>
                                        <span>"S"</span>
                                        <span>"S"</span>
                                    </h2>
                                </CornerFrame>
                            </div>
                            <div class="flex flex-col gap-2 flex-1">
                                {move || {
                                    let items = passkeys.get();
                                    if items.is_empty() {
                                        view! {
                                            <p class="text-xs text-base-content/60 border border-dashed border-base-content/20 rounded-none p-2">
                                                "No passkeys yet. Add one to sign in with your fingerprint, face, or device PIN."
                                            </p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <ul class="flex flex-col gap-1">
                                                {items.into_iter().map(|pk| {
                                                    let display = if pk.name.is_empty() {
                                                        format!("Passkey ({})", format_time_full(pk.created_at))
                                                    } else {
                                                        format!("{} ({})", pk.name, format_time_full(pk.created_at))
                                                    };
                                                    let last = pk.last_used_at.map(format_time_full);
                                                    let id = pk.id.clone();
                                                    view! {
                                                        <li class="flex items-center justify-between gap-2 border border-base-content/20 p-2 text-xs">
                                                            <div class="flex flex-col min-w-0">
                                                                <span class="truncate text-base-content/80">{display}</span>
                                                                {last.map(|l| view! {
                                                                    <span class="text-base-content/50">{format!("Last used {l}")}</span>
                                                                })}
                                                            </div>
                                                            <button
                                                                type="button"
                                                                class="btn btn-ghost btn-xs text-error"
                                                                on:click=move |_| delete_passkey_at(id.clone())
                                                                aria-label="Remove passkey"
                                                            >
                                                                "Remove"
                                                            </button>
                                                        </li>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </ul>
                                        }.into_any()
                                    }
                                }}
                                <Show when=move || can_passkey>
                                    {move || {
                                        if naming_passkey.get() {
                                            view! {
                                                <form
                                                    class="flex flex-col gap-2"
                                                    on:submit=move |ev| {
                                                        ev.prevent_default();
                                                        if !passkey_in_flight.get() {
                                                            start_register_passkey(());
                                                        }
                                                    }
                                                >
                                                    <input
                                                        type="text"
                                                        class="input input-sm w-full rounded-none"
                                                        placeholder="Name this passkey"
                                                        maxlength="64"
                                                        prop:disabled=passkey_in_flight.get()
                                                        prop:value=passkey_name
                                                        on:input=move |ev| set_passkey_name.set(event_target_value(&ev))
                                                    />
                                                    <div class="flex gap-2">
                                                        <button
                                                            type="submit"
                                                            class="btn btn-primary btn-sm flex-1 rounded-none"
                                                            prop:disabled=passkey_in_flight.get()
                                                        >
                                                            {if passkey_in_flight.get() { "Adding passkey..." } else { "Add" }}
                                                        </button>
                                                        <button
                                                            type="button"
                                                            class="btn btn-ghost btn-sm rounded-none"
                                                            prop:disabled=passkey_in_flight.get()
                                                            on:click=cancel_naming_passkey
                                                        >
                                                            "Cancel"
                                                        </button>
                                                    </div>
                                                </form>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <button
                                                    type="button"
                                                    class="btn btn-outline btn-sm w-full justify-start gap-2 rounded-none"
                                                    on:click=start_naming_passkey
                                                >
                                                    <svg class="w-4 h-4" viewBox="0 0 256 256" fill="currentColor" aria-hidden="true">
                                                        <path d="M216.57,39.43A80,80,0,0,0,83.91,120.78L28.69,176A15.86,15.86,0,0,0,24,187.31V216a16,16,0,0,0,16,16H72a8,8,0,0,0,8-8V208H96a8,8,0,0,0,8-8V184h16a8,8,0,0,0,5.66-2.34l9.56-9.57A79.73,79.73,0,0,0,160,176h.1A80,80,0,0,0,216.57,39.43ZM224,98.1c-1.09,34.09-29.75,61.86-63.89,61.9H160a63.7,63.7,0,0,1-23.65-4.51,8,8,0,0,0-8.84,1.68L116.69,168H96a8,8,0,0,0-8,8v16H72a8,8,0,0,0-8,8v16H40V187.31l58.83-58.82a8,8,0,0,0,1.68-8.84A63.72,63.72,0,0,1,96,95.92c0-34.14,27.81-62.8,61.9-63.89A64,64,0,0,1,224,98.1ZM192,76a12,12,0,1,1-12-12A12,12,0,0,1,192,76Z"/>
                                                    </svg>
                                                    "Add passkey"
                                                </button>
                                            }.into_any()
                                        }
                                    }}
                                </Show>
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

/// Fetches the provider redirect URL that starts the OAuth linking flow.
///
/// Authenticated variant of the login flow's URL fetch: the session cookie is
/// sent so the backend can associate the provider with the current account.
async fn fetch_link_url(url: &str) -> Result<String, String> {
    let resp = Request::get(url)
        .credentials(web_sys::RequestCredentials::Include)
        .send()
        .await
        .map_err(|err| format!("request failed: {err}"))?;
    if !resp.ok() {
        return Err(format!("server returned status {}", resp.status()));
    }
    let parsed = resp
        .json::<AuthUrlResponse>()
        .await
        .map_err(|err| format!("invalid response: {err}"))?;
    Ok(parsed.url)
}

#[derive(Debug, serde::Deserialize)]
struct AuthUrlResponse {
    url: String,
}
