use crate::data::{CardData, fetch_cards};
use leptos::prelude::*;

/// Provides the list of projects fetched from the backend.
#[derive(Clone, Copy)]
pub struct ProjectListContext {
    pub cards: Signal<Vec<CardData>>,
    pub set_cards: WriteSignal<Vec<CardData>>,
}

impl ProjectListContext {
    /// Provide an empty card list and kick off a fetch from `/api/projects`.
    pub fn provide() {
        let (cards, set_cards) = signal(Vec::new());
        provide_context(Self {
            cards: cards.into(),
            set_cards,
        });

        leptos::task::spawn_local(async move {
            let fetched = fetch_cards().await;
            set_cards.set(fetched);
        });
    }

    /// Read the current context, panicking if none was provided.
    pub fn use_context() -> Self {
        leptos::prelude::expect_context::<Self>()
    }
}
