use crate::contexts::LoginModalContext;
use crate::utils::{encode_redirect_url, login_url, window_event_listener};
use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::web_sys;

use super::search::SearchModal;

#[derive(Debug, serde::Deserialize)]
struct AuthUrlResponse {
    url: String,
}

/// Starts the OAuth flow for the given provider by fetching the provider URL
/// from the backend and navigating the browser there.
async fn start_oauth(provider: &str) {
    let url = encode_redirect_url(&login_url(provider));
    if let Ok(resp) = Request::get(&url).send().await
        && let Ok(parsed) = resp.json::<AuthUrlResponse>().await
        && let Some(window) = leptos::web_sys::window()
    {
        let _ = window.location().set_href(&parsed.url);
    }
}

/// Modal dialog that offers OAuth login choices.
#[component]
pub fn LoginModal() -> impl IntoView {
    let modal = LoginModalContext::use_context();
    let on_close = move |_| modal.close();

    // Keyboard shortcuts: Alt+1 triggers GitHub login, Alt+2 triggers Google
    // login, but only while the modal is open.
    Effect::new(move |_| {
        let modal = modal;
        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if !ev.alt_key() || !modal.open.get_untracked() {
                return;
            }
            if ev.key() == "1" {
                ev.prevent_default();
                leptos::task::spawn_local(async move { start_oauth("/github").await });
            } else if ev.key() == "2" {
                ev.prevent_default();
                leptos::task::spawn_local(async move { start_oauth("/google").await });
            }
        });
    });

    view! {
        <SearchModal
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
                    "Sign in with your GitHub or Google account to create and manage projects."
                </p>

                <div class="flex flex-col gap-3">
                    <button
                        type="button"
                        class="btn btn-outline btn-lift w-full justify-start gap-3"
                        on:click=move |_| leptos::task::spawn_local(async move { start_oauth("/github").await })
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z" />
                        </svg>
                        "Continue with GitHub"
                        <kbd class="hidden sm:inline-flex ml-auto px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            "Alt + 1"
                        </kbd>
                    </button>

                    <button
                        type="button"
                        class="btn btn-outline btn-lift w-full justify-start gap-3"
                        on:click=move |_| leptos::task::spawn_local(async move { start_oauth("/google").await })
                    >
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z" />
                            <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" />
                            <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" />
                            <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" />
                        </svg>
                        "Continue with Google"
                        <kbd class="hidden sm:inline-flex ml-auto px-1.5 py-0.5 text-xs font-sans font-semibold text-white bg-black/10 border border-black/30 rounded shadow-kbd">
                            "Alt + 2"
                        </kbd>
                    </button>
                </div>

                <p class="text-xs text-base-content/60 text-center">
                    "We only request public profile and email information."
                </p>
            </div>
        </SearchModal>
    }
}
