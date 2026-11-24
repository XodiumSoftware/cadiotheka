use crate::components::ui::icons::{GitHubMarkIcon, GoogleMarkIcon, PasskeyIcon};
use crate::contexts::{LoginModalContext, ToastContext};
use crate::utils::{
    LAST_LOGIN_METHOD_KEY, PENDING_LOGIN_METHOD_KEY, encode_redirect_url, github_login_url,
    google_login_url, key as storage_key, login_with_passkey, passkey_supported,
    window_event_listener,
};
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, SessionStorage, Storage as _};
use leptos::prelude::*;
use leptos::web_sys;

use super::base::BaseModal;

#[derive(Debug, serde::Deserialize)]
struct AuthUrlResponse {
    url: String,
}

/// Fetches the provider redirect URL that starts the OAuth flow.
async fn fetch_auth_url(url: &str) -> Result<String, String> {
    let resp = Request::get(url)
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

/// Starts the OAuth flow for the given provider by fetching the provider URL
/// from the backend and navigating the browser there. Shows a toast when the
/// flow cannot be started.
async fn start_oauth(provider: &str, toast: ToastContext) {
    let method = match provider {
        "/github" => "github",
        "/google" => "google",
        _ => return,
    };
    let url = match provider {
        "/github" => encode_redirect_url(&github_login_url()),
        "/google" => encode_redirect_url(&google_login_url()),
        _ => return,
    };
    match fetch_auth_url(&url).await {
        Ok(target) => {
            crate::utils::save::<SessionStorage>(&storage_key(PENDING_LOGIN_METHOD_KEY), &method);
            crate::utils::navigate_to(&target);
        }
        Err(err) => {
            leptos::web_sys::console::error_1(&format!("Failed to start login: {err}").into());
            toast.show("Couldn't start login. Please try again.");
        }
    }
}

/// Renders a "Last used" chip for the button matching the current
/// `last_login_method` in `localStorage`. Renders nothing when the value is
/// missing or refers to a different method.
#[component]
fn LastUsedBadge(method: &'static str, last: StoredValue<Option<String>>) -> impl IntoView {
    let is_match = move || last.get_value().as_deref() == Some(method);
    view! {
        <Show when=is_match>
            <span class="badge badge-xs badge-primary rounded-none font-sans uppercase tracking-wider">
                "Last used"
            </span>
        </Show>
    }
}

/// Modal dialog that offers OAuth and passkey login choices.
#[component]
pub fn LoginModal() -> impl IntoView {
    let modal = LoginModalContext::use_context();
    let toast = ToastContext::use_context();
    let on_close = move |()| modal.close();

    let (passkey_in_flight, set_passkey_in_flight) = signal(false);
    let can_passkey = passkey_supported();

    let last_method =
        StoredValue::new(LocalStorage::get::<String>(&storage_key(LAST_LOGIN_METHOD_KEY)).ok());

    let start_passkey = move || {
        let toast = toast;
        let modal = modal;
        set_passkey_in_flight.set(true);
        leptos::task::spawn_local(async move {
            let redirect_to = leptos::web_sys::window()
                .and_then(|w| w.location().href().ok())
                .unwrap_or_else(|| "/".to_string());
            match login_with_passkey(&redirect_to).await {
                Ok(()) => {
                    crate::utils::save::<LocalStorage>(
                        &storage_key(LAST_LOGIN_METHOD_KEY),
                        &"passkey",
                    );
                    modal.close();
                }
                Err(err) if err == "Sign-in cancelled" => {
                    set_passkey_in_flight.set(false);
                }
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Passkey sign-in failed: {err}").into(),
                    );
                    toast.show(&err);
                    set_passkey_in_flight.set(false);
                }
            }
        });
    };

    Effect::new(move |_| {
        let modal = modal;
        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if !ev.alt_key() || !modal.open.get_untracked() {
                return;
            }
            if ev.key() == "1" {
                ev.prevent_default();
                leptos::task::spawn_local(async move { start_oauth("/github", toast).await });
            } else if ev.key() == "2" {
                ev.prevent_default();
                leptos::task::spawn_local(async move { start_oauth("/google", toast).await });
            } else if ev.key() == "3" && can_passkey {
                ev.prevent_default();
                start_passkey();
            }
        });
    });

    view! {
        <BaseModal
            open=modal.open
            on_close=on_close
        >
            <div class="space-y-6 flex flex-col min-h-0">
                <div class="flex items-center justify-between">
                    <h2 class="text-xl font-bold text-primary">"Log in to Cadiotheka"</h2>
                    <div class="hidden sm:flex items-center gap-1.5 text-xs text-base-content/50">
                        <kbd class="px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            "esc"
                        </kbd>
                        <span>"to dismiss"</span>
                    </div>
                </div>

                <p class="text-sm text-base-content/80">
                    "Sign in with your GitHub, Google, or passkey to create and manage projects."
                </p>

                <div class="flex flex-col gap-3">
                    <button
                        type="button"
                        class="btn btn-outline btn-lift w-full justify-start gap-3"
                        aria-label="Continue with GitHub"
                        on:click=move |_| leptos::task::spawn_local(async move { start_oauth("/github", toast).await })
                    >
                        <GitHubMarkIcon class="w-5 h-5" />
                        "GitHub"
                        <span class="sr-only">"Continue with GitHub"</span>
                        <span class="ml-auto flex items-center gap-2">
                            <LastUsedBadge method="github" last=last_method />
                            <kbd class="hidden sm:inline-flex px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                                "Alt + 1"
                            </kbd>
                        </span>
                    </button>

                    <button
                        type="button"
                        class="btn btn-outline btn-lift w-full justify-start gap-3"
                        aria-label="Continue with Google"
                        on:click=move |_| leptos::task::spawn_local(async move { start_oauth("/google", toast).await })
                    >
                        <GoogleMarkIcon class="w-5 h-5" />
                        "Google"
                        <span class="sr-only">"Continue with Google"</span>
                        <span class="ml-auto flex items-center gap-2">
                            <LastUsedBadge method="google" last=last_method />
                            <kbd class="hidden sm:inline-flex px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                                "Alt + 2"
                            </kbd>
                        </span>
                    </button>

                    <Show when=move || can_passkey>
                        <button
                            type="button"
                            class="btn btn-outline btn-lift w-full justify-start gap-3"
                            aria-label="Continue with passkey"
                            prop:disabled=passkey_in_flight.get()
                            on:click=move |_| start_passkey()
                        >
                            <PasskeyIcon class="w-5 h-5" />
                            {if passkey_in_flight.get() { "Waiting..." } else { "Passkey" }}
                            <span class="sr-only">"Continue with passkey"</span>
                            <span class="ml-auto flex items-center gap-2">
                                <LastUsedBadge method="passkey" last=last_method />
                                <kbd class="hidden sm:inline-flex px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                                    "Alt + 3"
                                </kbd>
                            </span>
                        </button>
                    </Show>
                </div>

                <p class="text-xs text-base-content/60 text-center">
                    "We only request public profile and email information."
                </p>
            </div>
        </BaseModal>
    }
}
