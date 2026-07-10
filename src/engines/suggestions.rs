//! Autocomplete suggestion generation for the Cadiotheka search engine.

use crate::components::card::CardData;
use crate::engines::query::{SortBy, SortOrder};
use std::collections::HashSet;

/// Kind of suggestion shown in the search bar popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionKind {
    /// A sort directive such as `@sort:downloads:ascending`.
    Sort,
    /// An author filter such as `ZenFlow` (displayed as `@author:ZenFlow`).
    Author,
    /// A tag or platform filter such as `Blender` (displayed as `#Blender`).
    Filter,
    /// A plain search term such as a title or author.
    Plain,
}

/// A single clickable search suggestion.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Suggestion {
    /// Display and insert text (without any prefix).
    pub text: String,
    /// Kind of suggestion, controlling rendering and insertion behavior.
    pub kind: SuggestionKind,
}

impl Suggestion {
    /// Creates a new sort directive suggestion.
    pub fn sort(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: SuggestionKind::Sort,
        }
    }

    /// Creates a new filter suggestion.
    pub fn filter(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: SuggestionKind::Filter,
        }
    }

    /// Creates a new author filter suggestion.
    pub fn author(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: SuggestionKind::Author,
        }
    }

    /// Creates a new plain search suggestion.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: SuggestionKind::Plain,
        }
    }
}

/// Generates clickable suggestions from a list of cards.
pub fn from_cards(cards: &[CardData]) -> Vec<Suggestion> {
    let mut titles = HashSet::new();
    let mut authors = HashSet::new();
    let mut tags = HashSet::new();
    let mut platforms = HashSet::new();

    for card in cards {
        titles.insert(card.title.clone());
        authors.insert(card.author.clone());
        for tag in &card.tags {
            tags.insert(tag.label().to_owned());
        }
        for platform in &card.supported_platforms {
            platforms.insert(platform.label().to_owned());
        }
    }

    let mut suggestions = default_sort_suggestions();

    let mut push_sorted = |set: HashSet<String>, kind: SuggestionKind| {
        let mut items: Vec<String> = set.into_iter().collect();
        items.sort_by_key(|a| a.to_lowercase());
        for item in items {
            suggestions.push(Suggestion { text: item, kind });
        }
    };

    push_sorted(titles, SuggestionKind::Plain);
    push_sorted(authors, SuggestionKind::Author);
    push_sorted(tags, SuggestionKind::Filter);
    push_sorted(platforms, SuggestionKind::Filter);

    suggestions
}

/// Default sort directive suggestions.
fn default_sort_suggestions() -> Vec<Suggestion> {
    let mut suggestions = Vec::new();
    for by in SortBy::all() {
        for order in [SortOrder::Ascending, SortOrder::Descending] {
            suggestions.push(Suggestion::sort(format!(
                "@sort:{}:{}",
                by.label(),
                order.label()
            )));
        }
    }
    suggestions
}
