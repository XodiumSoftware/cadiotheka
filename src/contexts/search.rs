use leptos::prelude::*;

/// Provides and reads the live search state shared between the header modal
/// and the project grid.
#[derive(Clone, Copy)]
pub struct SearchContext {
    pub query: Signal<String>,
    pub set_query: WriteSignal<String>,
}

impl SearchContext {
    /// Provide a default empty search context.
    pub fn provide_with_default() {
        let (query, set_query) = signal(String::new());
        provide_context(Self {
            query: query.into(),
            set_query,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
