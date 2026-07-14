//! Application-wide reactive contexts for Cadiotheka.

use leptos::prelude::*;

const LAYOUT_WIDE_KEY: &str = "cadiotheka.layout_wide";

fn load_layout_wide() -> Option<bool> {
    let storage = leptos::web_sys::window()?.local_storage().ok().flatten()?;
    let value = storage.get_item(LAYOUT_WIDE_KEY).ok().flatten()?;
    Some(value == "true")
}

fn save_layout_wide(wide: bool) {
    if let Some(window) = leptos::web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item(LAYOUT_WIDE_KEY, if wide { "true" } else { "false" });
    }
}

/// Provides and reads the wide/narrow grid layout preference.
///
/// `true` = wide (5 columns), `false` = narrow (3 columns).
#[derive(Clone, Copy)]
pub struct LayoutContext {
    pub wide: Signal<bool>,
    pub set_wide: WriteSignal<bool>,
}

impl LayoutContext {
    /// Create a provider context, reading any persisted preference from
    /// `localStorage` and falling back to `default` if none exists.
    pub fn provide_with_default(default: bool) {
        let initial = load_layout_wide().unwrap_or(default);
        let (wide, set_wide) = signal(initial);
        provide_context(Self {
            wide: wide.into(),
            set_wide,
        });

        Effect::new(move |_| {
            save_layout_wide(wide.get());
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
