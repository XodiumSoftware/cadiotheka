use crate::components::{
    AddProjectModal, Footer, Header, LoginModal, ProfileModal, ProjectModal, ProjectsSection,
};
use crate::contexts::{
    AccountsContext, AddProjectModalContext, CurrentUserContext, LayoutContext, LoginModalContext,
    ProfileModalContext, ProjectModalContext, ProjectsContext, SearchContext,
};
use crate::utils::window_event_listener;
use leptos::prelude::*;
use leptos::web_sys;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <InnerApp />
    }
}

#[component]
fn InnerApp() -> impl IntoView {
    LayoutContext::provide_with_default(false);
    AccountsContext::provide();
    ProjectsContext::provide();
    SearchContext::provide_with_default();
    ProjectModalContext::provide_with_default();
    ProfileModalContext::provide_with_default();
    LoginModalContext::provide_with_default();
    AddProjectModalContext::provide_with_default();
    CurrentUserContext::provide();

    let layout = LayoutContext::use_context();
    let add_project_modal = AddProjectModalContext::use_context();
    let current_user = CurrentUserContext::use_context();

    Effect::new(move |_| {
        let layout = layout;
        let add_project_modal = add_project_modal;
        let current_user = current_user;
        window_event_listener::<web_sys::KeyboardEvent, _>("keydown", move |ev| {
            if ev.alt_key() && ev.key().eq_ignore_ascii_case("w") {
                let wide_enough = web_sys::window()
                    .and_then(|w| w.inner_width().ok())
                    .and_then(|w| w.as_f64())
                    .is_some_and(|width| width >= 1920.0);
                if wide_enough {
                    ev.prevent_default();
                    layout.set_wide.set(!layout.wide.get_untracked());
                }
                return;
            }

            if ev.alt_key() && ev.key().eq_ignore_ascii_case("n") {
                ev.prevent_default();
                if current_user.account.get_untracked().is_some() {
                    add_project_modal.open();
                }
            }
        });
    });

    view! {
        <div class="min-h-screen flex flex-col">
            <a
                href="#main-content"
                class="sr-only focus:not-sr-only focus:absolute focus:top-4 focus:left-4 focus:z-50 focus:px-4 focus:py-2 focus:bg-primary focus:text-primary-content focus:rounded"
            >
                "Skip to main content"
            </a>
            <Header />

            <ProjectModal />
            <ProfileModal />
            <LoginModal />
            <AddProjectModal />

            <main id="main-content" tabindex="-1" class="flex-1 flex flex-col">
                <ProjectsSection class="flex-1" />
            </main>

            <Footer />
        </div>
    }
}
