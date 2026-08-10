use crate::utils::{local_storage_get, local_storage_set};
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
    const KEY: &str = "layout_wide";

    fn load() -> Option<bool> {
        local_storage_get(Self::KEY).map(|value| value == "true")
    }

    fn save(wide: bool) {
        local_storage_set(Self::KEY, if wide { "true" } else { "false" });
    }

    /// Create a provider context, reading any persisted preference from
    /// `localStorage` and falling back to `default` if none exists.
    pub fn provide_with_default(default: bool) {
        let initial = Self::load().unwrap_or(default);
        let (wide, set_wide) = signal(initial);
        provide_context(Self {
            wide: wide.into(),
            set_wide,
        });

        Effect::new(move |_| {
            Self::save(wide.get());
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
