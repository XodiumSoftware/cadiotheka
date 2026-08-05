#![allow(clippy::missing_errors_doc)]

use crate::data::fetch_tags;
use crate::metadata::tags::Tag;
use leptos::prelude::*;
use std::collections::HashMap;

/// Provides the available tags fetched from the backend.
///
/// Projects reference these by wire id; components and the search engine look
/// up labels and colors from this context.
#[derive(Clone, Copy)]
pub struct MetadataContext {
    pub tags: Signal<Vec<Tag>>,
    pub set_tags: WriteSignal<Vec<Tag>>,
    pub is_loading: Signal<bool>,
    pub set_is_loading: WriteSignal<bool>,
}

impl MetadataContext {
    /// Provide empty metadata and kick off a fetch from `/data/tags`.
    pub fn provide() {
        let (tags, set_tags) = signal(Vec::new());
        let (is_loading, set_is_loading) = signal(true);
        provide_context(Self {
            tags: tags.into(),
            set_tags,
            is_loading: is_loading.into(),
            set_is_loading,
        });

        leptos::task::spawn_local(async move {
            match fetch_tags().await {
                Ok(fetched) => set_tags.set(fetched),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to load tags: {}", err.message()).into(),
                    );
                }
            }
            set_is_loading.set(false);
        });
    }

    /// Refetch tags from the backend and update the signals.
    pub fn refresh(&self) {
        let set_tags = self.set_tags;
        let set_is_loading = self.set_is_loading;
        leptos::task::spawn_local(async move {
            set_is_loading.set(true);
            match fetch_tags().await {
                Ok(fetched) => set_tags.set(fetched),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to refresh tags: {}", err.message()).into(),
                    );
                }
            }
            set_is_loading.set(false);
        });
    }

    /// Returns the tag record matching an id, or `None` if it is not loaded.
    pub fn tag_by_id(&self, id: &str) -> Option<Tag> {
        self.tags.get().into_iter().find(|t| t.id == id)
    }

    /// Builds an id-to-label map for tags, used by the search engine.
    pub fn tag_labels(&self) -> HashMap<String, String> {
        self.tags
            .get()
            .into_iter()
            .map(|t| (t.id, t.label))
            .collect()
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
