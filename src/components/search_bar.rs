//! Search bar widget for the Cadiotheka hub.
//!
//! The search bar supports:
//!
//! - Inline sort directives: `@sort:downloads:ascending` or `@sort:favorites:desc`.
//! - Exact category/platform filters: `#Blender`, `#FreeCAD`.
//! - Free-text search across titles, authors, descriptions, tags and platforms.
//!
//! When focused, a popup shows clickable suggestions: sort directives when the
//! query contains `@`, tag/platform filters when it contains `#`, and plain
//! card-derived terms otherwise.

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
    fn parse(token: &str) -> Option<Self> {
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

/// Combined sort selection returned by the search bar.
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

/// State and rendering for a search control bar.
#[derive(Default)]
pub struct SearchBar {
    /// Current raw search query.
    pub query: String,
}

impl SearchBar {
    /// Draws the search input and returns the parsed query.
    ///
    /// When focused, a suggestion popup is shown below the input. Clicking a
    /// suggestion appends it to the query.
    pub fn show(&mut self, ui: &mut egui::Ui, suggestions: &[Suggestion]) -> ParsedQuery {
        let response = ui
            .add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text(i18n::SearchBar::PLACEHOLDER)
                    .margin(egui::vec2(16.0, 12.0))
                    .desired_width(f32::INFINITY),
            )
            .on_hover_text("Use @sort:field:direction to sort, #tag to filter");

        let popup = egui::Popup::from_response(&response)
            .open_memory(
                response
                    .gained_focus()
                    .then_some(egui::SetOpenCommand::Bool(true)),
            )
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .layout(egui::Layout::top_down_justified(egui::Align::Min));

        popup.show(|ui| self.render_suggestions(ui, suggestions));

        Self::parse_query(&self.query)
    }

    /// Renders clickable suggestion rows inside the popup.
    fn render_suggestions(&mut self, ui: &mut egui::Ui, suggestions: &[Suggestion]) {
        let filtered = self.filtered_suggestions(suggestions);
        if filtered.is_empty() {
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for suggestion in filtered {
                    let label = match suggestion.kind {
                        SuggestionKind::Sort => egui::RichText::new(&suggestion.text).monospace(),
                        SuggestionKind::Filter => {
                            egui::RichText::new(format!("#{}", suggestion.text))
                        }
                        SuggestionKind::Plain => egui::RichText::new(&suggestion.text),
                    };

                    let button = ui.button(label);
                    if button.clicked() {
                        self.apply_suggestion(&suggestion);
                        egui::Popup::close_all(ui.ctx());
                    }
                }
            });
    }

    /// Returns suggestions filtered by the current query context.
    fn filtered_suggestions(&self, suggestions: &[Suggestion]) -> Vec<Suggestion> {
        let prefix = self.active_prefix();
        let needle = self.active_needle();
        let needle_lower = needle.to_lowercase();

        suggestions
            .iter()
            .filter(|s| {
                if let Some(prefix) = prefix {
                    match s.kind {
                        SuggestionKind::Sort if prefix == '@' => {
                            s.text.to_lowercase().contains(&needle_lower)
                        }
                        SuggestionKind::Filter if prefix == '#' => {
                            s.text.to_lowercase().contains(&needle_lower)
                        }
                        _ => false,
                    }
                } else {
                    s.kind == SuggestionKind::Plain && s.text.to_lowercase().contains(&needle_lower)
                }
            })
            .take(8)
            .cloned()
            .collect()
    }

    /// Determines which prefix context the user is currently typing in.
    ///
    /// Returns `'@'` if the last token starts with `@`, `'#'` if it starts with
    /// `#`, or `None` for plain text.
    fn active_prefix(&self) -> Option<char> {
        let last = self.query.split_whitespace().last()?;
        if last.starts_with('@') {
            Some('@')
        } else if last.starts_with('#') {
            Some('#')
        } else {
            None
        }
    }

    /// Returns the current completion needle within the active prefix token.
    fn active_needle(&self) -> String {
        self.query
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

    /// Appends a clicked suggestion to the query, replacing the partial token
    /// when completing a prefixed suggestion.
    fn apply_suggestion(&mut self, suggestion: &Suggestion) {
        match suggestion.kind {
            SuggestionKind::Sort => {
                let mut parts: Vec<&str> = self.query.split_whitespace().collect();
                parts.retain(|p| !p.starts_with("@sort:"));
                parts.push(&suggestion.text);
                self.query = parts.join(" ");
            }
            SuggestionKind::Filter => {
                let replacement = format!("#{}", suggestion.text);
                self.replace_last_token(&replacement);
            }
            SuggestionKind::Plain => {
                let needs_space = !self.query.is_empty() && !self.query.ends_with(' ');
                if needs_space {
                    self.query.push(' ');
                }
                self.query.push_str(&suggestion.text);
            }
        }
    }

    /// Replaces the last whitespace-separated token with a new string.
    fn replace_last_token(&mut self, replacement: &str) {
        let mut parts: Vec<&str> = self.query.split_whitespace().collect();
        if parts.is_empty() {
            self.query = replacement.to_owned();
            return;
        }
        parts.pop();
        parts.push(replacement);
        self.query = parts.join(" ");
    }

    /// Generates the default list of sort directive suggestions.
    pub fn default_sort_suggestions() -> Vec<Suggestion> {
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

    /// Parses the raw query into filter text, filters, and an optional sort directive.
    fn parse_query(query: &str) -> ParsedQuery {
        let mut filter_parts = Vec::new();
        let mut filters = Vec::new();
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
}

/// Kind of suggestion shown in the popup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionKind {
    /// A sort directive such as `@sort:downloads:ascending`.
    Sort,
    /// A tag or platform filter such as `Blender` (displayed as `#Blender`).
    Filter,
    /// A plain search term such as a title or author.
    Plain,
}

/// A single clickable suggestion.
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

    /// Creates a new plain search suggestion.
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            kind: SuggestionKind::Plain,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_query() {
        let parsed = SearchBar::parse_query("");
        assert_eq!(parsed.filter, "");
        assert!(parsed.filters.is_empty());
        assert_eq!(parsed.sort.by, SortBy::Downloads);
        assert_eq!(parsed.sort.order, SortOrder::Ascending);
    }

    #[test]
    fn parse_filter_only() {
        let parsed = SearchBar::parse_query("parametric screw");
        assert_eq!(parsed.filter, "parametric screw");
        assert!(parsed.filters.is_empty());
    }

    #[test]
    fn parse_tag_and_platform_filters() {
        let parsed = SearchBar::parse_query("screw #Blender #FreeCAD");
        assert_eq!(parsed.filter, "screw");
        assert_eq!(parsed.filters, vec!["blender", "freecad"]);
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

    #[test]
    fn active_prefix_detects_hash() {
        let mut bar = SearchBar {
            query: "screw #Ble".to_owned(),
        };
        assert_eq!(bar.active_prefix(), Some('#'));
        assert_eq!(bar.active_needle(), "Ble".to_lowercase());

        bar.query = "@sort:down".to_owned();
        assert_eq!(bar.active_prefix(), Some('@'));
        assert_eq!(bar.active_needle(), "sort:down".to_lowercase());

        bar.query = "parametric".to_owned();
        assert_eq!(bar.active_prefix(), None);
        assert_eq!(bar.active_needle(), "parametric");
    }

    #[test]
    fn apply_filter_replaces_partial_token() {
        let mut bar = SearchBar {
            query: "screw #Ble".to_owned(),
        };
        bar.apply_suggestion(&Suggestion::filter("Blender"));
        assert_eq!(bar.query, "screw #Blender");
    }
}
