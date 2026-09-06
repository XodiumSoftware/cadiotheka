use crate::data::{ProjectData, RequestError, fetch_projects, toggle_project_favorite};
use leptos::prelude::*;

/// Provides the list of projects fetched from the backend.
#[derive(Clone, Copy)]
pub struct ProjectsContext {
    pub projects: Signal<Vec<ProjectData>>,
    pub set_projects: WriteSignal<Vec<ProjectData>>,
    pub is_loading: Signal<bool>,
    pub set_is_loading: WriteSignal<bool>,
    /// The last fetch failure, or `None` when the list loaded successfully.
    pub error: Signal<Option<RequestError>>,
    set_error: WriteSignal<Option<RequestError>>,
}

impl ProjectsContext {
    /// Toggle the current user's favorite status for a project and return the
    /// updated project on success.
    ///
    /// # Errors
    ///
    /// Returns a [`RequestError`] when the backend request fails.
    pub async fn toggle_favorite(id: &str) -> Result<ProjectData, RequestError> {
        toggle_project_favorite(id).await
    }

    /// Spawns a fetch of `/data/projects` that updates the list, loading flag,
    /// and error state on completion.
    fn spawn_fetch(
        set_projects: WriteSignal<Vec<ProjectData>>,
        set_is_loading: WriteSignal<bool>,
        set_error: WriteSignal<Option<RequestError>>,
    ) {
        leptos::task::spawn_local(async move {
            set_is_loading.set(true);
            set_error.set(None);
            match fetch_projects().await {
                Ok(fetched) => set_projects.set(fetched),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to load projects: {}", err.message()).into(),
                    );
                    set_error.set(Some(err));
                }
            }
            set_is_loading.set(false);
        });
    }

    /// Provide an empty project list and kick off a fetch from `/data/projects`.
    pub fn provide() {
        let (projects, set_projects) = signal(Vec::new());
        let (is_loading, set_is_loading) = signal(true);
        let (error, set_error) = signal(None);
        provide_context(Self {
            projects: projects.into(),
            set_projects,
            is_loading: is_loading.into(),
            set_is_loading,
            error: error.into(),
            set_error,
        });

        Self::spawn_fetch(set_projects, set_is_loading, set_error);
    }

    /// Re-fetches the project list after a failure.
    pub fn retry(&self) {
        Self::spawn_fetch(self.set_projects, self.set_is_loading, self.set_error);
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
