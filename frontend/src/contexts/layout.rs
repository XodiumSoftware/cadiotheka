use crate::utils::key as storage_key;
use gloo_storage::{LocalStorage, Storage as _};
use leptos::prelude::*;

/// Provides and reads the wide/narrow grid layout preference.
///
/// `true` = wide (5 columns), `false` = narrow (3 columns).
#[derive(Clone, Copy)]
pub struct LayoutContext {
    /// Whether the grid uses the wide layout; `true` means five columns.
    pub wide: Signal<bool>,
    /// Setter that switches between the wide and narrow layouts.
    pub set_wide: WriteSignal<bool>,
}

impl LayoutContext {
    const KEY: &str = "layout_wide";

    fn load() -> Option<bool> {
        LocalStorage::get::<bool>(&storage_key(Self::KEY)).ok()
    }

    fn save(wide: bool) {
        let _ = LocalStorage::set(storage_key(Self::KEY), wide);
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
        expect_context::<Self>()
    }
}
