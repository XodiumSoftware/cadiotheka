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
    Ascending,
    /// Descending order (highest first).
    #[default]
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

/// Parsed search query, borrowing from the original query string.
///
/// Using borrowed `&str` slices avoids allocating a new `String` for every
/// token on every keystroke.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedQuery<'a> {
    /// Free-text filter tokens with directives and filters removed.
    pub filter: Vec<&'a str>,
    /// Exact tag/platform filters requested with `#`.
    pub filters: Vec<&'a str>,
    /// Author filter requested with `@author:`.
    pub author: Option<&'a str>,
    /// Account id filter for projects favorited by that account.
    pub favorited_by: Option<&'a str>,
    /// Sort selection parsed from directives.
    pub sort: SortSelection,
    /// Whether the user explicitly requested a sort directive.
    ///
    /// When `false`, results are ranked by fuzzy match score instead.
    pub sort_explicit: bool,
}

/// Parses a raw query string into a structured [`ParsedQuery`].
pub fn parse_query(query: &str) -> ParsedQuery<'_> {
    let mut filter_parts = Vec::new();
    let mut filters = Vec::new();
    let mut author: Option<&str> = None;
    let mut favorited_by: Option<&str> = None;
    let mut sort = SortSelection::default();
    let mut sort_found = false;

    for token in query.split_whitespace() {
        if token.starts_with('@') {
            if !sort_found && let Some(parsed) = parse_sort_directive(token) {
                sort = parsed;
                sort_found = true;
                continue;
            }
            if let Some(value) = token.strip_prefix("@author:") {
                if !value.is_empty() {
                    author = Some(value);
                }
                continue;
            }
            if let Some(value) = token.strip_prefix("@favorited_by:") {
                if !value.is_empty() {
                    favorited_by = Some(value);
                }
                continue;
            }
            continue;
        }

        if let Some(filter) = token.strip_prefix('#') {
            filters.push(filter);
            continue;
        }

        filter_parts.push(token);
    }

    ParsedQuery {
        filter: filter_parts,
        filters,
        author,
        favorited_by,
        sort,
        sort_explicit: sort_found,
    }
}

/// Returns the current completion needle within the active prefix token.
///
/// For prefixed tokens such as `#model` or `@sort:down`, the prefix is stripped
/// so the matcher can rank the actual label. Plain tokens are returned as-is.
pub fn active_needle(query: &str) -> String {
    query
        .split_whitespace()
        .last()
        .map(|token| {
            if token.starts_with('@') || token.starts_with('#') {
                token[1..].to_owned()
            } else {
                token.to_owned()
            }
        })
        .unwrap_or_default()
}

/// Parses a single `@sort:<field>:<direction>` directive.
fn parse_sort_directive(token: &str) -> Option<SortSelection> {
    let body = token.strip_prefix("@sort:")?;
    let parts: Vec<&str> = body.split(':').collect();
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
        assert!(parsed.filter.is_empty());
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Descending);
    }

    #[test]
    fn parse_filter_only() {
        let parsed = parse_query("parametric screw");
        assert_eq!(parsed.filter, vec!["parametric", "screw"]);
        assert!(parsed.filters.is_empty());
    }

    #[test]
    fn parse_tag_and_platform_filters() {
        let parsed = parse_query("screw #Blender #FreeCAD");
        assert_eq!(parsed.filter, vec!["screw"]);
        assert_eq!(parsed.filters, vec!["Blender", "FreeCAD"]);
    }

    #[test]
    fn parse_sort_directive() {
        let parsed = parse_query("screw @sort:favorites:descending");
        assert_eq!(parsed.filter, vec!["screw"]);
        assert_eq!(parsed.sort.by, SortBy::Favorites);
        assert_eq!(parsed.sort.order, SortOrder::Descending);
    }

    #[test]
    fn parse_sort_short_direction() {
        let parsed = parse_query("@sort:newest:asc");
        assert!(parsed.filter.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Newest);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn invalid_sort_token_ignored() {
        let parsed = parse_query("gear @sort:rating:descending");
        assert_eq!(parsed.filter, vec!["gear"]);
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Descending);
    }

    #[test]
    fn incomplete_sort_token_ignored() {
        let parsed = parse_query("gear @sort:downloads:in");
        assert_eq!(parsed.filter, vec!["gear"]);
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Descending);
    }

    #[test]
    fn only_first_sort_directive_used() {
        let parsed = parse_query("@sort:favorites:asc @sort:downloads:desc");
        assert!(parsed.filter.is_empty());
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Favorites);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn parse_author_filter_borrows_slice() {
        let query = "screw @author:zenflow";
        let parsed = parse_query(query);
        assert_eq!(parsed.filter, vec!["screw"]);
        assert_eq!(parsed.author, Some("zenflow"));
    }
}
