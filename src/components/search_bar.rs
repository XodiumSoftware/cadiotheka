//! Search bar UI widget for the Cadiotheka hub.
//!
//! This file only handles rendering and user interaction. All query parsing,
//! filtering, scoring, and suggestion logic lives in [`crate::engines`].

use crate::engines::{ParsedQuery, Suggestion, SuggestionKind};
use crate::i18n;

/// State and rendering for a search control bar.
pub struct SearchBar {
    /// Current raw search query.
    pub query: String,
    /// Persistent widget id used to request focus programmatically.
    pub id: egui::Id,
    /// Currently selected suggestion index in the popup, if any.
    selected_suggestion: Option<usize>,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self {
            query: String::new(),
            id: egui::Id::new("hub_search_bar"),
            selected_suggestion: None,
        }
    }
}

impl SearchBar {
    /// Draws the search input and returns the parsed query.
    ///
    /// When focused, a suggestion popup is shown below the input. Suggestions
    /// can be selected with the mouse, or with arrow keys and Enter.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        suggestions: &[Suggestion],
        parse: impl FnOnce(&str) -> ParsedQuery,
    ) -> ParsedQuery {
        let response = ui
            .add(
                egui::TextEdit::singleline(&mut self.query)
                    .id(self.id)
                    .hint_text(i18n::SearchBar::PLACEHOLDER)
                    .margin(egui::vec2(16.0, 12.0))
                    .desired_width(f32::INFINITY),
            )
            .on_hover_text("Use @sort:field:direction to sort, #tag to filter");

        let filtered = self.filtered_suggestions(suggestions);

        if response.changed() || filtered.is_empty() {
            self.selected_suggestion = filtered.first().map(|_| 0);
        }

        self.handle_keyboard(ui, &filtered);

        let popup = egui::Popup::from_response(&response)
            .open_memory(
                response
                    .changed()
                    .then_some(egui::SetOpenCommand::Bool(true)),
            )
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .layout(egui::Layout::top_down_justified(egui::Align::Min))
            .width(response.rect.width())
            .frame(
                egui::Frame::new()
                    .inner_margin(0.0)
                    .fill(ui.visuals().panel_fill)
                    .corner_radius(6.0),
            );

        let popup_width = response.rect.width();
        let selected = self.selected_suggestion;
        popup.show(|ui| {
            ui.set_min_width(popup_width);
            ui.set_max_width(popup_width);
            self.render_suggestions(ui, &filtered, selected)
        });

        parse(&self.query)
    }

    /// Handles arrow-key navigation, Enter selection, and Escape closing.
    fn handle_keyboard(&mut self, ui: &mut egui::Ui, filtered: &[Suggestion]) {
        if filtered.is_empty() {
            return;
        }

        let count = filtered.len();
        let mut changed = false;

        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.selected_suggestion = Some(
                self.selected_suggestion
                    .map(|i| (i + 1) % count)
                    .unwrap_or(0),
            );
            changed = true;
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.selected_suggestion = Some(
                self.selected_suggestion
                    .map(|i| i.saturating_sub(1))
                    .unwrap_or(count - 1),
            );
            changed = true;
        }

        if changed {
            return;
        }

        if ui.input(|i| i.key_pressed(egui::Key::Enter))
            && let Some(index) = self.selected_suggestion
            && let Some(suggestion) = filtered.get(index)
        {
            self.apply_suggestion(suggestion);
            self.selected_suggestion = None;
            egui::Popup::close_all(ui.ctx());
        }

        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_suggestion = None;
            egui::Popup::close_all(ui.ctx());
        }
    }

    /// Renders clickable suggestion rows inside the popup.
    fn render_suggestions(
        &mut self,
        ui: &mut egui::Ui,
        suggestions: &[Suggestion],
        selected: Option<usize>,
    ) {
        if suggestions.is_empty() {
            return;
        }

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for (index, suggestion) in suggestions.iter().enumerate() {
                    let is_selected = selected == Some(index);
                    let label = match suggestion.kind {
                        SuggestionKind::Sort => egui::RichText::new(&suggestion.text).monospace(),
                        SuggestionKind::Author => {
                            egui::RichText::new(format!("@author:{}", suggestion.text))
                        }
                        SuggestionKind::Filter => {
                            egui::RichText::new(format!("#{}", suggestion.text))
                        }
                        SuggestionKind::Plain => egui::RichText::new(&suggestion.text),
                    };

                    let item = ui.selectable_label(is_selected, label);
                    if item.clicked() {
                        self.apply_suggestion(suggestion);
                        self.selected_suggestion = None;
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
                        SuggestionKind::Author if prefix == '@' => {
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

                if parts.last().is_some_and(|p| p.starts_with('@')) {
                    parts.pop();
                }

                parts.push(&suggestion.text);
                self.query = parts.join(" ");
            }
            SuggestionKind::Author => {
                let replacement = format!("@author:{}", suggestion.text);
                self.replace_last_token(&replacement);
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_prefix_detects_hash() {
        let mut bar = SearchBar {
            query: "screw #Ble".to_owned(),
            id: egui::Id::new("test_search_bar"),
            selected_suggestion: None,
        };
        assert_eq!(bar.active_prefix(), Some('#'));
        assert_eq!(bar.active_needle(), "Ble");

        bar.query = "@sort:down".to_owned();
        assert_eq!(bar.active_prefix(), Some('@'));
        assert_eq!(bar.active_needle(), "sort:down");

        bar.query = "parametric".to_owned();
        assert_eq!(bar.active_prefix(), None);
        assert_eq!(bar.active_needle(), "parametric");
    }

    #[test]
    fn apply_sort_replaces_partial_token() {
        let mut bar = SearchBar {
            query: "screw @so".to_owned(),
            id: egui::Id::new("test_search_bar"),
            selected_suggestion: None,
        };
        bar.apply_suggestion(&Suggestion::sort("@sort:downloads:ascending"));
        assert_eq!(bar.query, "screw @sort:downloads:ascending");
    }

    #[test]
    fn apply_filter_replaces_partial_token() {
        let mut bar = SearchBar {
            query: "screw #Ble".to_owned(),
            id: egui::Id::new("test_search_bar"),
            selected_suggestion: None,
        };
        bar.apply_suggestion(&Suggestion::filter("Blender"));
        assert_eq!(bar.query, "screw #Blender");
    }

    #[test]
    fn apply_filter_replaces_lonely_partial_token() {
        let mut bar = SearchBar {
            query: "#Ble".to_owned(),
            id: egui::Id::new("test_search_bar"),
            selected_suggestion: None,
        };
        bar.apply_suggestion(&Suggestion::filter("Blender"));
        assert_eq!(bar.query, "#Blender");
    }
}
