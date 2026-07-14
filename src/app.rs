use crate::components::{
    Footer, Header, LandingSection, LineDraw, ProjectsSection, TeamDeckSection,
};
use crate::i18n::{I18nContextProvider, t, use_i18n};
use leptos::prelude::*;

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
    view! {
        <div>
            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-content focus:rounded"
            >
                {t!(i18n, skip_to_content)}
            </a>
            <Header />

        <main id="main-content" tabindex="-1">
        <LandingSection />

        // Line divider between hero and projects
        <LineDraw />

        // Projects section
        <ProjectsSection />

        // Line draw divider
        <LineDraw />

        // Team section
        <TeamDeckSection />
        </main>

        <Footer />
    </div>
    }
}
