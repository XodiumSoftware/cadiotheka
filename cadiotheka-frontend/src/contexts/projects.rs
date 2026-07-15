use crate::data::{ProjectData, fetch_projects};
use leptos::prelude::*;

/// Provides the list of projects fetched from the backend.
#[derive(Clone, Copy)]
pub struct ProjectsContext {
    pub projects: Signal<Vec<ProjectData>>,
    pub set_projects: WriteSignal<Vec<ProjectData>>,
}

impl ProjectsContext {
    /// Provide an empty project list and kick off a fetch from `/api/projects`.
    pub fn provide() {
        let (projects, set_projects) = signal(Vec::new());
        provide_context(Self {
            projects: projects.into(),
            set_projects,
        });

        leptos::task::spawn_local(async move {
            let fetched = fetch_projects().await;
            leptos::web_sys::console::log_1(&format!("Projects fetched: {}", fetched.len()).into());
            set_projects.set(fetched);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
