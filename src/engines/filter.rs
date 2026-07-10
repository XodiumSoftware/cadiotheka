//! Fuzzy matching and card filtering for the Cadiotheka search engine.

use crate::components::card::CardData;
use crate::engines::query::{ParsedQuery, SortBy, SortOrder, parse_query};
use crate::engines::suggestions::{Suggestion, from_cards};
use crate::tags::Tag;
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Search engine that owns the loaded cards and answers search queries.
pub struct SearchEngine {
    cards: Vec<CardData>,
    matcher: SkimMatcherV2,
}

impl SearchEngine {
    /// Creates a new search engine from a list of cards.
    pub fn new(cards: Vec<CardData>) -> Self {
        Self {
            cards,
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Returns cards matching the parsed query, ranked or sorted as requested.
    pub fn search(&self, parsed: &ParsedQuery) -> Vec<CardData> {
        let query = parsed.filter.trim().to_lowercase();

        let mut scored: Vec<(i64, CardData)> = self
            .cards
            .iter()
            .filter_map(|card| {
                let score = self.score(card, &query, &parsed.filters)?;
                Some((score, card.clone()))
            })
            .collect();

        let use_fuzzy_rank =
            parsed.sort.by == SortBy::default() && parsed.sort.order == SortOrder::default();

        if use_fuzzy_rank {
            scored.sort_by_key(|a| std::cmp::Reverse(a.0));
        } else {
            scored.sort_by(|a, b| match parsed.sort.by {
                SortBy::Downloads => match parsed.sort.order {
                    SortOrder::Ascending => a.1.downloads.cmp(&b.1.downloads),
                    SortOrder::Descending => b.1.downloads.cmp(&a.1.downloads),
                },
                SortBy::Favorites => match parsed.sort.order {
                    SortOrder::Ascending => a.1.favorites.cmp(&b.1.favorites),
                    SortOrder::Descending => b.1.favorites.cmp(&a.1.favorites),
                },
                SortBy::Newest => match parsed.sort.order {
                    SortOrder::Ascending => a.1.timestamp.cmp(&b.1.timestamp),
                    SortOrder::Descending => b.1.timestamp.cmp(&a.1.timestamp),
                },
            });
        }

        scored.into_iter().map(|(_, card)| card).collect()
    }

    /// Returns a fuzzy match score for a card, or `None` if it does not match.
    fn score(&self, card: &CardData, query: &str, filters: &[String]) -> Option<i64> {
        let matches_filters = filters.iter().all(|filter| {
            card.tags
                .iter()
                .any(|tag| tag.label().to_lowercase().starts_with(filter))
                || card
                    .supported_platforms
                    .iter()
                    .any(|platform| platform.label().to_lowercase().starts_with(filter))
        });
        if !matches_filters {
            return None;
        }

        if query.is_empty() {
            return Some(0);
        }

        let haystack = Self::searchable_text(card);
        self.matcher.fuzzy_match(&haystack, query)
    }

    /// Generates clickable suggestions for the search bar popup.
    pub fn suggestions(&self) -> Vec<Suggestion> {
        from_cards(&self.cards)
    }

    /// Parses a raw query string into a structured [`ParsedQuery`].
    pub fn parse_query(query: &str) -> ParsedQuery {
        parse_query(query)
    }

    /// Combines all searchable card fields into a single lowercase string.
    fn searchable_text(card: &CardData) -> String {
        let tags = card
            .tags
            .iter()
            .map(Tag::label)
            .collect::<Vec<_>>()
            .join(" ");
        let platforms = card
            .supported_platforms
            .iter()
            .map(|platform| platform.label())
            .collect::<Vec<_>>()
            .join(" ");
        format!(
            "{} {} {} {} {}",
            card.title, card.author, card.description, tags, platforms
        )
        .to_lowercase()
    }
}
