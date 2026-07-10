//! Query parsing for the Cadiotheka search engine.

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
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "downloads" => Some(Self::Downloads),
            "favorites" => Some(Self::Favorites),
            "newest" => Some(Self::Newest),
            _ => None,
        }
    }

    /// All available sort fields.
    pub const fn all() -> &'static [Self] {
        &[Self::Downloads, Self::Favorites, Self::Newest]
    }

    /// User-facing label for the sort field.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Downloads => "downloads",
            Self::Favorites => "favorites",
            Self::Newest => "newest",
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
    pub fn parse(token: &str) -> Option<Self> {
        match token {
            "asc" | "ascending" => Some(Self::Ascending),
            "desc" | "descending" => Some(Self::Descending),
            _ => None,
        }
    }

    /// Long form of the direction name.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }

    /// Short form of the direction name.
    pub const fn short(self) -> &'static str {
        match self {
            Self::Ascending => "asc",
            Self::Descending => "desc",
        }
    }
}

/// Combined sort selection.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SortSelection {
    /// Field to sort by.
    pub by: SortBy,
    /// Direction of the sort.
    pub order: SortOrder,
}

/// Parsed search query.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedQuery {
    /// Free-text filter with directives and filters removed.
    pub filter: String,
    /// Exact tag/platform filters requested with `#`.
    pub filters: Vec<String>,
    /// Sort selection parsed from directives.
    pub sort: SortSelection,
}

/// Parses a raw query string into a structured [`ParsedQuery`].
pub fn parse_query(query: &str) -> ParsedQuery {
    let mut filter_parts = Vec::new();
    let mut filters = Vec::new();
    let mut sort = SortSelection::default();
    let mut sort_found = false;

    for token in query.split_whitespace() {
        if token.starts_with('@') {
            if !sort_found && let Some(parsed) = parse_sort_directive(token) {
                sort = parsed;
                sort_found = true;
            }
            continue;
        }

        if let Some(filter) = token.strip_prefix('#') {
            filters.push(filter.to_lowercase());
            continue;
        }

        filter_parts.push(token);
    }

    ParsedQuery {
        filter: filter_parts.join(" "),
        filters,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_query() {
        let parsed = parse_query("");
        assert_eq!(parsed.filter, "");
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn parse_filter_only() {
        let parsed = parse_query("parametric screw");
        assert_eq!(parsed.filter, "parametric screw");
        assert!(parsed.filters.is_empty());
    }

    #[test]
    fn parse_tag_and_platform_filters() {
        let parsed = parse_query("screw #Blender #FreeCAD");
        assert_eq!(parsed.filter, "screw");
        assert_eq!(parsed.filters, vec!["blender", "freecad"]);
    }

    #[test]
    fn parse_sort_directive() {
        let parsed = parse_query("screw @sort:favorites:descending");
        assert_eq!(parsed.filter, "screw");
        assert_eq!(parsed.sort.by, SortBy::Favorites);
        assert_eq!(parsed.sort.order, SortOrder::Descending);
    }

    #[test]
    fn parse_sort_short_direction() {
        let parsed = parse_query("@sort:newest:asc");
        assert_eq!(parsed.filter, "");
        assert_eq!(parsed.sort.by, SortBy::Newest);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn invalid_sort_token_ignored() {
        let parsed = parse_query("gear @sort:rating:descending");
        assert_eq!(parsed.filter, "gear");
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn incomplete_sort_token_ignored() {
        let parsed = parse_query("gear @sort:downloads:desc");
        assert_eq!(parsed.filter, "gear");
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn only_first_sort_directive_used() {
        let parsed = parse_query("@sort:favorites:asc @sort:downloads:desc");
        assert_eq!(parsed.filter, "");
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Favorites);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }
}
