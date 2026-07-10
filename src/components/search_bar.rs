//! Search bar widget for the Cadiotheka hub.
//!
//! The search bar supports inline sort directives such as `@sort:downloads:ascending`
//! or `@sort:favorites:descending`. These directives are stripped from the visible
//! query and returned as structured sort state.

use crate::i18n;

/// Field to sort cards by.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortBy {
    /// Sort by download count.
    #[default]
    Downloads,
    /// Sort by favorite count.
    Favorites,
    /// Sort by timestamp.
    Newest,
}

impl SortBy {
    /// Parses a sort field from a raw token.
    fn parse(token: &str) -> Option<Self> {
        match token {
            "downloads" => Some(Self::Downloads),
            "favorites" => Some(Self::Favorites),
            "newest" => Some(Self::Newest),
            _ => None,
        }
    }
}

/// Direction of the sort order.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SortOrder {
    /// Ascending order (lowest first).
    #[default]
    Ascending,
    /// Descending order (highest first).
    Descending,
}

impl SortOrder {
    /// Parses a sort direction from a raw token.
    fn parse(token: &str) -> Option<Self> {
        match token {
            "asc" | "ascending" => Some(Self::Ascending),
            "desc" | "descending" => Some(Self::Descending),
            _ => None,
        }
    }
}

/// Combined sort selection returned by the search bar.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SortSelection {
    /// Field to sort by.
    pub by: SortBy,
    /// Direction of the sort.
    pub order: SortOrder,
}

/// Parsed search query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQuery {
    /// Filter text with sort directives removed.
    pub filter: String,
    /// Sort selection parsed from directives.
    pub sort: SortSelection,
}

/// State and rendering for a search control bar.
#[derive(Default)]
pub struct SearchBar {
    /// Current raw search query.
    pub query: String,
}

impl SearchBar {
    /// Draws the search input and returns the parsed query and sort selection.
    pub fn show(&mut self, ui: &mut egui::Ui) -> ParsedQuery {
        ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .hint_text(i18n::SearchBar::PLACEHOLDER)
                .margin(egui::vec2(16.0, 12.0)),
        );

        Self::parse_query(&self.query)
    }

    /// Parses the raw query into filter text and an optional sort directive.
    ///
    /// The first `@sort:<field>:<direction>` token is used; everything else is
    /// treated as filter text. Invalid sort tokens are ignored, falling back to
    /// the default sort.
    fn parse_query(query: &str) -> ParsedQuery {
        let mut filter_parts = Vec::new();
        let mut sort = SortSelection::default();
        let mut sort_found = false;

        for token in query.split_whitespace() {
            if !sort_found
                && token.starts_with("@sort:")
                && let Some(parsed) = Self::parse_sort_directive(token)
            {
                sort = parsed;
                sort_found = true;
                continue;
            }
            filter_parts.push(token);
        }

        ParsedQuery {
            filter: filter_parts.join(" "),
            sort,
        }
    }

    /// Parses a single `@sort:<field>:<direction>` directive.
    fn parse_sort_directive(token: &str) -> Option<SortSelection> {
        let parts: Vec<&str> = token[6..].split(':').collect();
        if parts.len() != 2 {
            return None;
        }

        let by = SortBy::parse(parts[0].to_lowercase().as_str())?;
        let order = SortOrder::parse(parts[1].to_lowercase().as_str())?;
        Some(SortSelection { by, order })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_query() {
        let parsed = SearchBar::parse_query("");
        assert_eq!(parsed.filter, "");
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn parse_filter_only() {
        let parsed = SearchBar::parse_query("parametric screw");
        assert_eq!(parsed.filter, "parametric screw");
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn parse_sort_directive() {
        let parsed = SearchBar::parse_query("screw @sort:favorites:descending");
        assert_eq!(parsed.filter, "screw");
        assert_eq!(parsed.sort.by, SortBy::Favorites);
        assert_eq!(parsed.sort.order, SortOrder::Descending);
    }

    #[test]
    fn parse_sort_short_direction() {
        let parsed = SearchBar::parse_query("@sort:newest:asc");
        assert_eq!(parsed.filter, "");
        assert_eq!(parsed.sort.by, SortBy::Newest);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn invalid_sort_token_ignored() {
        let parsed = SearchBar::parse_query("gear @sort:rating:descending");
        assert_eq!(parsed.filter, "gear @sort:rating:descending");
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn only_first_sort_directive_used() {
        let parsed = SearchBar::parse_query("@sort:favorites:asc @sort:downloads:desc");
        assert_eq!(parsed.filter, "@sort:downloads:desc");
        assert_eq!(parsed.sort.by, SortBy::Favorites);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }
}
