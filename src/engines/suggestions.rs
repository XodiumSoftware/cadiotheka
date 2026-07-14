//! Autocomplete suggestion generation for the Cadiotheka search engine.

use crate::data::CardData;
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
///
/// When `include_sort` is `false`, sort directives are omitted from the
/// returned list. This keeps the default suggestion popup free of sort
/// noise until the user explicitly types `@`.
pub fn from_cards(cards: &[CardData], include_sort: bool) -> Vec<Suggestion> {
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

    let mut suggestions = if include_sort {
        default_sort_suggestions()
    } else {
        Vec::new()
    };

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CardData;
    use crate::metadata::platforms::Platform;
    use crate::metadata::tags::Tag;
    use time::macros::datetime;

    fn sample_card() -> CardData {
        CardData {
            title: "Sample Gear".to_owned(),
            author: "TestAuthor".to_owned(),
            description: "A sample gear.".to_owned(),
            tags: vec![Tag::Parametric, Tag::Model3d],
            supported_platforms: vec![Platform::Blender, Platform::FreeCAD],
            downloads: 100,
            favorites: 10,
            timestamp: datetime!(2024-06-01 10:00:00 UTC),
            icon_url: None,
        }
    }

    #[test]
    fn default_sort_suggestions_contains_all_combinations() {
        let suggestions = default_sort_suggestions();
        assert_eq!(suggestions.len(), 6);
        assert!(suggestions.iter().all(|s| s.kind == SuggestionKind::Sort));
        assert!(
            suggestions
                .iter()
                .any(|s| s.text == "@sort:downloads:ascending")
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text == "@sort:downloads:descending")
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text == "@sort:favorites:ascending")
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text == "@sort:favorites:descending")
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text == "@sort:newest:ascending")
        );
        assert!(
            suggestions
                .iter()
                .any(|s| s.text == "@sort:newest:descending")
        );
    }

    #[test]
    fn from_cards_extracts_unique_values() {
        let card = sample_card();
        let suggestions = from_cards(&[card.clone(), card], false);

        let plain: Vec<_> = suggestions
            .iter()
            .filter(|s| s.kind == SuggestionKind::Plain)
            .map(|s| s.text.clone())
            .collect();
        let authors: Vec<_> = suggestions
            .iter()
            .filter(|s| s.kind == SuggestionKind::Author)
            .map(|s| s.text.clone())
            .collect();
        let filters: Vec<_> = suggestions
            .iter()
            .filter(|s| s.kind == SuggestionKind::Filter)
            .map(|s| s.text.clone())
            .collect();

        assert_eq!(plain, vec!["Sample Gear"]);
        assert_eq!(authors, vec!["TestAuthor"]);
        assert!(filters.contains(&"Parametric".to_owned()));
        assert!(filters.contains(&"3D Model".to_owned()));
        assert!(filters.contains(&"Blender".to_owned()));
        assert!(filters.contains(&"FreeCAD".to_owned()));
    }

    #[test]
    fn from_cards_includes_sort_when_requested() {
        let suggestions = from_cards(&[sample_card()], true);
        let sort_count = suggestions
            .iter()
            .filter(|s| s.kind == SuggestionKind::Sort)
            .count();
        assert_eq!(sort_count, 6);
    }

    #[test]
    fn from_cards_excludes_sort_when_not_requested() {
        let suggestions = from_cards(&[sample_card()], false);
        assert!(!suggestions.iter().any(|s| s.kind == SuggestionKind::Sort));
    }

    #[test]
    fn from_cards_sorts_suggestions_case_insensitively() {
        let mut card_a = sample_card();
        card_a.title = "alpha".to_owned();
        let mut card_b = sample_card();
        card_b.title = "Beta".to_owned();

        let suggestions = from_cards(&[card_a, card_b], false);
        let plain: Vec<_> = suggestions
            .iter()
            .filter(|s| s.kind == SuggestionKind::Plain)
            .map(|s| s.text.clone())
            .collect();

        assert_eq!(plain, vec!["alpha", "Beta"]);
    }
}
