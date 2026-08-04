#![allow(clippy::missing_errors_doc)]

use crate::data::{fetch_platforms, fetch_tags};
use crate::metadata::platforms::Platform;
use crate::metadata::tags::Tag;
use leptos::prelude::*;
use std::collections::HashMap;

/// Provides the available tags and platforms fetched from the backend.
///
/// Projects reference these by wire id; components and the search engine look
/// up labels and colors from this context.
#[derive(Clone, Copy)]
pub struct MetadataContext {
    pub tags: Signal<Vec<Tag>>,
    pub set_tags: WriteSignal<Vec<Tag>>,
    pub platforms: Signal<Vec<Platform>>,
    pub set_platforms: WriteSignal<Vec<Platform>>,
    pub is_loading: Signal<bool>,
    pub set_is_loading: WriteSignal<bool>,
}

impl MetadataContext {
    /// Provide empty metadata and kick off fetches from `/data/tags` and
    /// `/data/platforms`.
    pub fn provide() {
        let (tags, set_tags) = signal(Vec::new());
        let (platforms, set_platforms) = signal(Vec::new());
        let (is_loading, set_is_loading) = signal(true);
        provide_context(Self {
            tags: tags.into(),
            set_tags,
            platforms: platforms.into(),
            set_platforms,
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
            match fetch_platforms().await {
                Ok(fetched) => set_platforms.set(fetched),
                Err(err) => {
                    leptos::web_sys::console::error_1(
                        &format!("Failed to load platforms: {}", err.message()).into(),
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

    /// Returns the platform record matching an id, or `None` if it is not loaded.
    pub fn platform_by_id(&self, id: &str) -> Option<Platform> {
        self.platforms.get().into_iter().find(|p| p.id == id)
    }

    /// Builds an id-to-label map for tags, used by the search engine.
    pub fn tag_labels(&self) -> HashMap<String, String> {
        self.tags
            .get()
            .into_iter()
            .map(|t| (t.id, t.label))
            .collect()
    }

    /// Builds an id-to-label map for platforms, used by the search engine.
    pub fn platform_labels(&self) -> HashMap<String, String> {
        self.platforms
            .get()
            .into_iter()
            .map(|p| (p.id, p.label))
            .collect()
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
