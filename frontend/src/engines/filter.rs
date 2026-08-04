//! Fuzzy matching and card filtering for the Cadiotheka search engine.

use crate::data::ProjectData;
use crate::engines::query::{ParsedQuery, SortBy, SortOrder, active_needle, parse_query};
use crate::engines::suggestions::{Suggestion, from_cards};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::collections::HashMap;

/// Search engine that owns the loaded cards and answers search queries.
///
/// Cards reference tags and platforms by wire id, so the engine also holds
/// id-to-label maps (built from `/data/tags` and `/data/platforms`) to resolve
/// user-facing labels for filtering and searchable text.
pub struct SearchEngine {
    cards: Vec<ProjectData>,
    matcher: SkimMatcherV2,
    tag_labels: HashMap<String, String>,
    platform_labels: HashMap<String, String>,
}

impl SearchEngine {
    /// Creates a new search engine from a list of projects and id-to-label maps
    /// for tags and platforms.
    pub fn new(
        cards: Vec<ProjectData>,
        tag_labels: HashMap<String, String>,
        platform_labels: HashMap<String, String>,
    ) -> Self {
        Self {
            cards,
            matcher: SkimMatcherV2::default(),
            tag_labels,
            platform_labels,
        }
    }

    /// Returns owned copies of projects matching the parsed query.
    ///
    /// Useful when results need to escape the borrow scope of the engine,
    /// such as in reactive Leptos memos.
    pub fn search_owned(&self, parsed: &ParsedQuery) -> Vec<ProjectData> {
        self.search(parsed).into_iter().cloned().collect()
    }

    /// Returns references to projects matching the parsed query, ranked or sorted
    /// as requested.
    ///
    /// Returning `&ProjectData` avoids cloning the owned project data on every
    /// search, which matters as the catalog grows.
    pub fn search<'a>(&'a self, parsed: &ParsedQuery) -> Vec<&'a ProjectData> {
        let query = parsed.filter.clone().join(" ").to_lowercase();

        let mut scored: Vec<(i64, &ProjectData)> = self
            .cards
            .iter()
            .filter_map(|card| {
                let score = self.score(
                    card,
                    &query,
                    &parsed.filters,
                    parsed.author,
                    parsed.favorited_by,
                )?;
                Some((score, card))
            })
            .collect();

        let use_fuzzy_rank = !parsed.sort_explicit;

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

    /// Returns a fuzzy match score for a project, or `None` if it does not match.
    fn score(
        &self,
        card: &ProjectData,
        query: &str,
        filters: &[&str],
        author: Option<&str>,
        favorited_by: Option<&str>,
    ) -> Option<i64> {
        let matches_filters = filters.iter().all(|filter| {
            card.tags.iter().any(|id| {
                self.tag_label(id)
                    .is_some_and(|label| Self::label_matches(label, filter))
            }) || card.platforms.iter().any(|id| {
                self.platform_label(id)
                    .is_some_and(|label| Self::label_matches(label, filter))
            })
        });
        if !matches_filters {
            return None;
        }

        if let Some(author) = author
            && !card
                .author_username
                .to_lowercase()
                .starts_with(author.to_lowercase().as_str())
        {
            return None;
        }

        if let Some(favorited_by) = favorited_by
            && !card.favorites.iter().any(|id| id == favorited_by)
        {
            return None;
        }

        if query.is_empty() {
            return Some(0);
        }

        let haystack = Self::searchable_text(card, &self.tag_labels, &self.platform_labels);
        self.matcher.fuzzy_match(&haystack, query)
    }

    /// Resolves a tag id to its user-facing label, or `None` if unknown.
    fn tag_label(&self, id: &str) -> Option<&str> {
        self.tag_labels.get(id).map(String::as_str)
    }

    /// Resolves a platform id to its user-facing label, or `None` if unknown.
    fn platform_label(&self, id: &str) -> Option<&str> {
        self.platform_labels.get(id).map(String::as_str)
    }

    /// Checks whether a user-facing label matches a filter needle.
    ///
    /// The label is tokenized on whitespace and non-alphanumeric characters,
    /// then each token is compared case-insensitively using substring
    /// matching. This lets `#model` match `3D Model` while still supporting
    /// prefixes like `#blend` -> `Blender`.
    fn label_matches(label: &str, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        label
            .split(|c: char| !c.is_alphanumeric())
            .map(str::to_lowercase)
            .any(|token| token.contains(&needle))
    }

    /// Generates clickable suggestions for the search bar popup.
    ///
    /// Suggestions are ranked by fuzzy relevance to the active completion needle.
    /// Sort suggestions are only included when the user is typing a `@` prefixed
    /// token, so the popup doesn't start with six sort directives on every focus.
    pub fn suggestions(&self, query: &str) -> Vec<Suggestion> {
        let include_sort = query
            .split_whitespace()
            .last()
            .is_some_and(|token| token.starts_with('@'));
        let needle = active_needle(query);
        from_cards(
            &self.cards,
            include_sort,
            &needle,
            &self.tag_labels,
            &self.platform_labels,
        )
    }

    /// Parses a raw query string into a structured [`ParsedQuery`].
    pub fn parse_query(query: &str) -> ParsedQuery<'_> {
        parse_query(query)
    }

    /// Combines all searchable project fields into a single lowercase string.
    fn searchable_text(
        card: &ProjectData,
        tag_labels: &HashMap<String, String>,
        platform_labels: &HashMap<String, String>,
    ) -> String {
        let tags = card
            .tags
            .iter()
            .filter_map(|id| tag_labels.get(id))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        let platforms = card
            .platforms
            .iter()
            .filter_map(|id| platform_labels.get(id))
            .cloned()
            .collect::<Vec<_>>()
            .join(" ");
        format!("{} {} {} {}", card.title, card.author, tags, platforms).to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::ProjectData;
    use crate::engines::query::parse_query;
    use crate::engines::suggestions::SuggestionKind;
    use time::macros::datetime;

    fn tag_labels() -> HashMap<String, String> {
        HashMap::from([
            ("3d_model".to_owned(), "3D Model".to_owned()),
            ("parametric".to_owned(), "Parametric".to_owned()),
            ("furniture".to_owned(), "Furniture".to_owned()),
            ("fabrication".to_owned(), "Fabrication".to_owned()),
            ("diy".to_owned(), "DIY".to_owned()),
            ("electronics".to_owned(), "Electronics".to_owned()),
            ("tooling".to_owned(), "Tooling".to_owned()),
        ])
    }

    fn platform_labels() -> HashMap<String, String> {
        HashMap::from([
            ("blender".to_owned(), "Blender".to_owned()),
            ("freecad".to_owned(), "FreeCAD".to_owned()),
            ("fusion_360".to_owned(), "Fusion 360".to_owned()),
            ("sketchup".to_owned(), "SketchUp".to_owned()),
            ("kicad".to_owned(), "KiCad".to_owned()),
        ])
    }

    #[allow(clippy::too_many_arguments)]
    fn card(
        title: &str,
        author: &str,
        author_username: &str,
        tags: &[&str],
        platforms: &[&str],
        downloads: u64,
        favorites: u64,
    ) -> ProjectData {
        let downloads_trunc = usize::try_from(downloads).unwrap_or(0);
        let favorites_trunc = usize::try_from(favorites).unwrap_or(0);

        ProjectData {
            id: format!(
                "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
                title.len() + 0x1000,
                author.len() + 0x2000,
                downloads_trunc % 0x1000,
                favorites_trunc % 0x10000,
                (downloads + favorites) % 0x1_0000_0000_0000
            ),
            title: title.to_owned(),
            author: author.to_owned(),
            author_id: format!(
                "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
                author.len() + 0x6000,
                0,
                0,
                0,
                0
            ),
            author_username: author_username.to_owned(),
            collaborator_ids: vec![],
            description: format!("Markdown summary for {title}."),
            tags: tags.iter().map(|s| (*s).to_owned()).collect(),
            platforms: platforms.iter().map(|s| (*s).to_owned()).collect(),
            downloads,
            favorites: vec!["favorite-user".to_owned(); favorites_trunc],
            timestamp: datetime!(2024-01-15 12:00:00 UTC),
            versions: vec![],
        }
    }

    fn engine() -> SearchEngine {
        SearchEngine::new(
            vec![
                card(
                    "Parametric Screw",
                    "ZenFlow",
                    "zenflow",
                    &["parametric", "3d_model"],
                    &["blender", "freecad", "fusion_360"],
                    1_200,
                    80,
                ),
                card(
                    "Workshop Bench",
                    "MakerJoe",
                    "makerjoe",
                    &["furniture", "fabrication", "diy"],
                    &["sketchup"],
                    3_400,
                    250,
                ),
                card(
                    "PCB Holder",
                    "ZenFlow",
                    "zenflow",
                    &["electronics", "tooling"],
                    &["kicad"],
                    900,
                    45,
                ),
            ],
            tag_labels(),
            platform_labels(),
        )
    }

    #[test]
    fn fuzzy_search_matches_title() {
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
        let parsed = parse_query("@author:zenflow");
        let results = engine.search(&parsed);
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|c| c.author_username == "zenflow"));
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
                .any(|s| { s.kind == SuggestionKind::Author && s.text == "zenflow" })
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
        assert_eq!(parsed.filter, vec!["gear"]);
        assert_eq!(parsed.filters, vec!["freecad"]);
    }

    #[test]
    fn filter_matches_substring_tokens() {
        let engine = engine();
        let parsed = parse_query("#model");
        let results = engine.search(&parsed);
        assert!(
            results.iter().any(|c| c.title == "Parametric Screw"),
            "#model should match the 3D Model tag"
        );

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
        assert!(std::ptr::eq(results[0], &raw const engine.cards[0]));
    }
}
