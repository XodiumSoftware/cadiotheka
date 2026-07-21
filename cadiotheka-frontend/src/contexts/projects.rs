use crate::data::{ProjectData, fetch_projects, toggle_project_favorite};
use leptos::prelude::*;

/// Provides the list of projects fetched from the backend.
#[derive(Clone, Copy)]
pub struct ProjectsContext {
    pub projects: Signal<Vec<ProjectData>>,
    pub set_projects: WriteSignal<Vec<ProjectData>>,
    pub is_loading: Signal<bool>,
    pub set_is_loading: WriteSignal<bool>,
}

impl ProjectsContext {
    /// Toggle the current user's favorite status for a project and return the
    /// updated project on success.
    pub async fn toggle_favorite(id: &str) -> Option<ProjectData> {
        toggle_project_favorite(id).await
    }

    /// Provide an empty project list and kick off a fetch from `/data/projects`.
    pub fn provide() {
        let (projects, set_projects) = signal(Vec::new());
        let (is_loading, set_is_loading) = signal(true);
        provide_context(Self {
            projects: projects.into(),
            set_projects,
            is_loading: is_loading.into(),
            set_is_loading,
        });

        leptos::task::spawn_local(async move {
            let fetched = fetch_projects().await;
            set_projects.set(fetched);
            set_is_loading.set(false);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
