//! Application-wide reactive contexts for Cadiotheka.

use leptos::prelude::*;

/// Provides and reads the wide/narrow grid layout preference.
///
/// `true` = wide (5 columns), `false` = narrow (3 columns).
#[derive(Clone, Copy)]
pub struct LayoutContext {
    pub wide: Signal<bool>,
    pub set_wide: WriteSignal<bool>,
}

impl LayoutContext {
    /// Create a provider context with a default value.
    pub fn provide_with_default(default: bool) {
        let (wide, set_wide) = signal(default);
        provide_context(Self {
            wide: wide.into(),
            set_wide,
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}

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
