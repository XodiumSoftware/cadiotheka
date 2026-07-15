use crate::components::ui::modals::search_modal::SearchModal;
use crate::contexts::ProfileModalContext;
use crate::i18n::{t_string, use_i18n};
use crate::utils::{placeholder_color, placeholder_letter};
use leptos::prelude::*;

/// Modal dialog that displays profile information for a selected account.
#[component]
pub fn ProfileModal() -> impl IntoView {
    let _i18n = use_i18n();
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
    let i18n = use_i18n();
    let letter = placeholder_letter(&account.username);
    let bg = placeholder_color(&account.username);
    let display_name = account.display_name.clone();
    let username = account.username.clone();
    let bio = account.bio.clone();
    let role_label = match account.role {
        crate::data::AccountRole::Creator => t_string!(i18n, account.role_creator),
        crate::data::AccountRole::Admin => t_string!(i18n, account.role_admin),
    };

    view! {
        <div class="space-y-0 flex flex-col min-h-0">
            <div class="flex items-start gap-4">
                <div class=format!("flex-shrink-0 w-16 h-16 rounded flex items-center justify-center text-white font-bold text-xl {}", bg)
                    aria-hidden="true"
                >
                    {letter}
                </div>
                <div class="min-w-0 flex-1 flex flex-col gap-1">
                    <h2 class="text-xl font-bold text-primary leading-tight truncate">
                        {display_name.clone()}
                    </h2>
                    <p class="text-base-content/70 text-sm">
                        {"@"}
                        {username.clone()}
                    </p>
                    <span class="badge badge-xs badge-outline rounded-none border-base-content/20 text-base-content/70 mt-1 self-start">
                        {role_label}
                    </span>
                </div>
            </div>
            <hr class="border-base-content/10 my-4" />
            <div class="space-y-2 text-sm text-base-content/80">
                <p>
                    <span class="font-semibold text-base-content">{t_string!(i18n, account.email_label)}</span>
                    <span class="ml-1">{account.email.clone()}</span>
                </p>
                <p>
                    <span class="font-semibold text-base-content">{t_string!(i18n, account.joined_label)}</span>
                    <span class="ml-1">{account.created_at.to_string()}</span>
                </p>
                {if bio.is_empty() {
                    None
                } else {
                    Some(view! {
                        <p class="text-base-content/70 mt-2">{bio}</p>
                    })
                }}
            </div>
            <hr class="border-base-content/10 my-4" />
            <div class="flex items-center justify-end">
                <button
                    type="button"
                    class="btn btn-primary btn-lift"
                    on:click=move |_| on_close.run(())
                >
                    {t_string!(i18n, account.close)}
                </button>
            </div>
        </div>
    }
}
