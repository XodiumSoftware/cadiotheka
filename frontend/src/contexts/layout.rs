use crate::utils::{local_storage_get, local_storage_set};
use leptos::prelude::*;

const LAYOUT_WIDE_KEY: &str = "layout_wide";

fn load_layout_wide() -> Option<bool> {
    local_storage_get(LAYOUT_WIDE_KEY).map(|value| value == "true")
}

fn save_layout_wide(wide: bool) {
    local_storage_set(LAYOUT_WIDE_KEY, if wide { "true" } else { "false" });
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
