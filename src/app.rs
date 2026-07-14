use crate::components::{Footer, Header, ProjectsSection};
use crate::context::LayoutContext;
use crate::i18n::{I18nContextProvider, t, use_i18n};
use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::web_sys;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <I18nContextProvider>
            <InnerApp />
        </I18nContextProvider>
    }
}

#[component]
fn InnerApp() -> impl IntoView {
    let i18n = use_i18n();
    LayoutContext::provide_with_default(false);

    Effect::new(move |_| {
        let layout = LayoutContext::use_context();
        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if ev.alt_key() && ev.key().eq_ignore_ascii_case("l") {
                ev.prevent_default();
                layout.set_wide.set(!layout.wide.get());
            }
        });
    });

    view! {
        <div class="min-h-screen flex flex-col">
            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-content focus:rounded"
            >
                {t!(i18n, skip_to_content)}
            </a>
            <Header />

            <main id="main-content" tabindex="-1" class="flex-1 flex flex-col">
                <ProjectsSection class="flex-1" />
            </main>

            <Footer />
        </div>
    }
}
