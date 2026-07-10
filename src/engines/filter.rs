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

    /// Returns references to cards matching the parsed query, ranked or sorted
    /// as requested.
    ///
    /// Returning `&CardData` avoids cloning the owned card data on every
    /// search, which matters as the catalog grows.
    pub fn search(&self, parsed: &ParsedQuery) -> Vec<&CardData> {
        let query = parsed.filter.trim().to_lowercase();

        let mut scored: Vec<(i64, &CardData)> = self
            .cards
            .iter()
            .filter_map(|card| {
                let score = self.score(card, &query, &parsed.filters, &parsed.author)?;
                Some((score, card))
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
    fn score(
        &self,
        card: &CardData,
        query: &str,
        filters: &[String],
        author: &Option<String>,
    ) -> Option<i64> {
        let matches_filters = filters.iter().all(|filter| {
            card.tags
                .iter()
                .any(|tag| Self::label_matches(tag.label(), filter))
                || card
                    .supported_platforms
                    .iter()
                    .any(|platform| Self::label_matches(platform.label(), filter))
        });
        if !matches_filters {
            return None;
        }

        if let Some(author) = author
            && !card.author.to_lowercase().starts_with(author)
        {
            return None;
        }

        if query.is_empty() {
            return Some(0);
        }

        let haystack = Self::searchable_text(card);
        self.matcher.fuzzy_match(&haystack, query)
    }

    /// Checks whether a user-facing label matches a filter needle.
    ///
    /// The label is tokenized on whitespace and non-alphanumeric characters,
    /// then each token is compared case-insensitively using substring
    /// matching. This lets `#model` match `3D Model` while still supporting
    /// prefixes like `#blend` → `Blender`.
    fn label_matches(label: &str, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        label
            .split(|c: char| !c.is_alphanumeric())
            .map(|token| token.to_lowercase())
            .any(|token| token.contains(&needle))
    }

    /// Generates clickable suggestions for the search bar popup.
    ///
    /// Sort suggestions are only included when the user is typing a `@` prefixed
    /// token, so the popup doesn't start with six sort directives on every
    /// focus.
    pub fn suggestions(&self, query: &str) -> Vec<Suggestion> {
        let include_sort = query
            .split_whitespace()
            .last()
            .is_some_and(|token| token.starts_with('@'));
        from_cards(&self.cards, include_sort)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::card::CardData;
    use crate::engines::query::parse_query;
    use crate::engines::suggestions::SuggestionKind;
    use crate::platforms::Platform;
    use crate::tags::Tag;
    use time::macros::datetime;

    fn card(
        title: &str,
        author: &str,
        description: &str,
        tags: &[Tag],
        platforms: &[Platform],
        downloads: u64,
        favorites: u64,
    ) -> CardData {
        CardData {
            title: title.to_owned(),
            author: author.to_owned(),
            description: description.to_owned(),
            tags: tags.to_vec(),
            supported_platforms: platforms.to_vec(),
            downloads,
            favorites,
            timestamp: datetime!(2024-01-15 12:00:00 UTC),
            icon_url: None,
        }
    }

    fn engine() -> SearchEngine {
        SearchEngine::new(vec![
            card(
                "Parametric Screw",
                "ZenFlow",
                "A fully parametric screw model.",
                &[Tag::Parametric, Tag::Model3d],
                &[Platform::Blender, Platform::FreeCAD],
                1_200,
                80,
            ),
            card(
                "Workshop Bench",
                "MakerJoe",
                "Sturdy bench for the garage.",
                &[Tag::Furniture, Tag::Fabrication, Tag::Diy],
                &[Platform::SketchUp],
                3_400,
                250,
            ),
            card(
                "PCB Holder",
                "ZenFlow",
                "Holder for KiCad projects.",
                &[Tag::Electronics, Tag::Tooling],
                &[Platform::KiCad],
                900,
                45,
            ),
        ])
    }

    #[test]
    fn empty_query_returns_all_cards() {
        let engine = engine();
        let parsed = parse_query("");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn fuzzy_search_matches_title_and_description() {
        let engine = engine();
        let parsed = parse_query("screw");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Parametric Screw");
    }

    #[test]
    fn tag_filter_excludes_non_matching_cards() {
        let engine = engine();
        let parsed = parse_query("#electronics");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "PCB Holder");
    }

    #[test]
    fn platform_filter_matches_card() {
        let engine = engine();
        let parsed = parse_query("#sketchup");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Workshop Bench");
    }

    #[test]
    fn author_filter_limits_results() {
        let engine = engine();
        let parsed = parse_query("@author:zen");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|c| c.author == "ZenFlow"));
    }

    #[test]
    fn combined_filter_and_text_query() {
        let engine = engine();
        let parsed = parse_query("holder #electronics");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "PCB Holder");
    }

    #[test]
    fn sort_by_downloads_descending() {
        let engine = engine();
        let parsed = parse_query("@sort:downloads:descending");
        let results = engine.search(&parsed);
        let titles: Vec<_> = results.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["Workshop Bench", "Parametric Screw", "PCB Holder"]
        );
    }

    #[test]
    fn sort_by_favorites_ascending() {
        let engine = engine();
        let parsed = parse_query("@sort:favorites:ascending");
        let results = engine.search(&parsed);
        let titles: Vec<_> = results.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(
            titles,
            vec!["PCB Holder", "Parametric Screw", "Workshop Bench"]
        );
    }

    #[test]
    fn sort_by_newest_uses_timestamp() {
        let engine = engine();
        let parsed = parse_query("@sort:newest:descending");
        let results = engine.search(&parsed);
        // All timestamps are identical in the fixture, so order is preserved by sort stability.
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn suggestions_derived_from_cards() {
        let engine = engine();
        let suggestions = engine.suggestions("");

        assert!(
            suggestions
                .iter()
                .any(|s| { s.kind == SuggestionKind::Plain && s.text == "Parametric Screw" })
        );
        assert!(
            suggestions
                .iter()
                .any(|s| { s.kind == SuggestionKind::Author && s.text == "ZenFlow" })
        );
        assert!(
            suggestions
                .iter()
                .any(|s| { s.kind == SuggestionKind::Filter && s.text == "Parametric" })
        );
        assert!(
            suggestions
                .iter()
                .any(|s| { s.kind == SuggestionKind::Filter && s.text == "Blender" })
        );
        assert!(
            !suggestions.iter().any(|s| s.kind == SuggestionKind::Sort),
            "sort suggestions should be hidden without @ prefix"
        );
    }

    #[test]
    fn sort_suggestions_shown_with_at_prefix() {
        let engine = engine();
        let suggestions = engine.suggestions("@");
        assert!(
            suggestions.iter().any(|s| s.kind == SuggestionKind::Sort),
            "sort suggestions should appear when @ prefix is active"
        );
    }

    #[test]
    fn parse_query_exposed_as_method() {
        let parsed = SearchEngine::parse_query("gear #freecad");
        assert_eq!(parsed.filter, "gear");
        assert_eq!(parsed.filters, vec!["freecad"]);
    }

    #[test]
    fn filter_matches_substring_tokens() {
        let engine = engine();
        // "3D Model" should match #model even though the label starts with "3D".
        let parsed = parse_query("#model");
        let results = engine.search(&parsed);
        assert!(
            results.iter().any(|c| c.title == "Parametric Screw"),
            "#model should match the 3D Model tag"
        );

        // "Fusion 360" should match #fusion (and #360).
        let parsed = parse_query("#fusion");
        let results = engine.search(&parsed);
        assert!(
            results.iter().any(|c| c.title == "Parametric Screw"),
            "#fusion should match the Fusion 360 platform"
        );
    }

    #[test]
    fn label_matches_tokenizes_and_substring_matches() {
        assert!(SearchEngine::label_matches("3D Model", "model"));
        assert!(SearchEngine::label_matches("3D Model", "3d"));
        assert!(SearchEngine::label_matches("Fusion 360", "fusion"));
        assert!(SearchEngine::label_matches("Blender", "blend"));
        assert!(!SearchEngine::label_matches("Blender", "model"));
    }

    #[test]
    fn search_returns_card_references_without_cloning() {
        let engine = engine();
        let parsed = parse_query("");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 3);
        // References should point to the engine's owned cards.
        assert!(std::ptr::eq(results[0], &engine.cards[0]));
    }
}
