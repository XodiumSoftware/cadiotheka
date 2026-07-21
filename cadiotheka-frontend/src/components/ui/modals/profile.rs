use crate::components::ui::modals::search::SearchModal;
use crate::contexts::ProfileModalContext;
use crate::utils::{format_time_full, placeholder_color, placeholder_letter};
use leptos::prelude::*;

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
                        <ProfileModalContent account=account on_close=on_close />
                    }
                })
            }}
        </SearchModal>
    }
}

#[component]
fn ProfileModalContent(
    #[prop(into)] account: crate::data::AccountData,
    #[prop(into)] on_close: Callback<()>,
) -> impl IntoView {
    let _ = on_close;
    let letter = placeholder_letter(&account.username);
    let bg = placeholder_color(&account.username);
    let display_name = account.display_name.clone();
    let username = account.username.clone();
    let bio = account.bio.clone();
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
                {if bio.is_empty() {
                    None
                } else {
                    Some(view! {
                        <p>
                            <span class="font-semibold text-base-content">Bio:</span>
                            <span class="ml-1 text-base-content/70">{bio}</span>
                        </p>
                    })
                }}
            </div>
        </div>
    }
}
