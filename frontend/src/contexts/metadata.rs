use crate::metadata::tags::Tag;
use leptos::prelude::*;
use std::collections::HashMap;

/// Provides hardcoded content-tag metadata to the component tree.
///
/// Tags used to be fetched from the backend, but they are now defined as an
/// enum in `frontend/src/metadata/tags.rs`. This context is kept so consumers
/// can still resolve tag labels and colors from wire ids without changing
/// every call site.
#[derive(Clone, Copy)]
pub struct MetadataContext {
    pub tags: Signal<Vec<Tag>>,
    pub is_loading: Signal<bool>,
}

impl MetadataContext {
    /// Provide the static tag list.
    pub fn provide() {
        let tags: Signal<Vec<Tag>> = Signal::derive(Tag::all);
        let (is_loading, set_is_loading) = signal(false);
        provide_context(Self {
            tags,
            is_loading: is_loading.into(),
        });
        set_is_loading.set(false);
    }

    /// Refetch is a no-op because tags are hardcoded. Kept for API compatibility.
    pub fn refresh(&self) {
        let _ = self;
    }

    /// Returns the tag matching an id, or `None` if the id is unknown.
    pub fn tag_by_id(&self, id: &str) -> Option<Tag> {
        Tag::from_id(id)
    }

    /// Builds an id-to-label map for tags, used by the search engine.
    pub fn tag_labels(&self) -> HashMap<String, String> {
        Tag::all()
            .into_iter()
            .map(|tag| (tag.id().to_string(), tag.label().to_string()))
            .collect()
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
