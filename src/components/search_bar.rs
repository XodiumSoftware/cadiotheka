//! Search bar UI widget for the Cadiotheka hub.
//!
//! This file only handles rendering and user interaction. All query parsing,
//! filtering, scoring, and suggestion logic lives in [`crate::engines`].

use crate::engines::{Suggestion, SuggestionKind};
use crate::i18n;

/// State and rendering for a search control bar.
pub struct SearchBar {
    /// Current raw search query.
    pub query: String,
    /// Persistent widget id used to request focus programmatically.
    pub id: egui::Id,
    /// Whether to request focus on the input during the next render.
    request_focus: bool,
    /// Currently selected suggestion index across the flattened dropdown list.
    selected_suggestion: Option<usize>,
}

impl Default for SearchBar {
    fn default() -> Self {
        Self {
            query: String::new(),
            id: egui::Id::new("hub_search_bar"),
            request_focus: false,
            selected_suggestion: None,
        }
    }
}

/// A group of suggestions shown under a category header in the dropdown.
struct SuggestionGroup {
    title: &'static str,
    suggestions: Vec<Suggestion>,
}

impl SearchBar {
    /// Requests focus for the search input on the next render.
    pub fn request_focus(&mut self) {
        self.request_focus = true;
    }

    /// Clears the query and selection.
    pub fn reset(&mut self) {
        self.query.clear();
        self.selected_suggestion = None;
    }

    /// Draws the search input and its inline category dropdown.
    ///
    /// Returns `true` when the user pressed Escape, signaling that the caller
    /// may want to close the containing modal.
    pub fn show(&mut self, ui: &mut egui::Ui, _query: &str, suggestions: &[Suggestion]) -> bool {
        let _response = self.render_input(ui);
        let applied = self.handle_input(ui, suggestions);
        self.render_dropdown(ui, suggestions, 320.0);
        self.wants_close(ui, applied)
    }

    /// Renders only the search input.
    pub fn render_input(&mut self, ui: &mut egui::Ui) -> egui::Response {
        let response = ui.add(
            egui::TextEdit::singleline(&mut self.query)
                .id(self.id)
                .hint_text(i18n::SearchBar::PLACEHOLDER)
                .margin(egui::vec2(16.0, 12.0))
                .desired_width(f32::INFINITY),
        );

        if self.request_focus {
            ui.memory_mut(|mem| mem.request_focus(self.id));
            self.request_focus = false;
        }

        response
    }

    /// Renders only the inline category dropdown.
    ///
    /// `max_height` limits the dropdown height. Use `f32::INFINITY` to let it
    /// fill the available space.
    pub fn render_dropdown(
        &mut self,
        ui: &mut egui::Ui,
        suggestions: &[Suggestion],
        max_height: f32,
    ) {
        let groups = self.grouped_suggestions(suggestions);

        if groups.iter().all(|group| group.suggestions.is_empty()) {
            return;
        }

        ui.add_space(8.0);

        let mut scroll_area = egui::ScrollArea::vertical();
        if max_height.is_finite() {
            scroll_area = scroll_area.max_height(max_height);
        }

        scroll_area.show(ui, |ui| {
            let mut global_index = 0usize;
            let mut selected_rect = None;
            let non_empty_indices: Vec<usize> = groups
                .iter()
                .enumerate()
                .filter(|(_, group)| !group.suggestions.is_empty())
                .map(|(index, _)| index)
                .collect();

            for (group_index, group) in groups.iter().enumerate() {
                if group.suggestions.is_empty() {
                    continue;
                }

                ui.label(egui::RichText::new(group.title).color(ui.visuals().weak_text_color()));

                for suggestion in &group.suggestions {
                    let is_selected = self.selected_suggestion == Some(global_index);
                    let label = self.suggestion_label(suggestion);

                    let item = ui.selectable_label(is_selected, label);
                    if is_selected {
                        selected_rect = Some(item.rect);
                    }
                    if item.clicked() {
                        self.apply_suggestion(suggestion);
                        self.selected_suggestion = None;
                    }

                    global_index += 1;
                }

                if non_empty_indices
                    .last()
                    .is_some_and(|last| *last != group_index)
                {
                    ui.separator();
                }
            }

            if let Some(rect) = selected_rect {
                ui.scroll_to_rect(rect, Some(egui::Align::Center));
            }
        });
    }

    /// Handles keyboard navigation and selection.
    ///
    /// Returns `true` if the user pressed Enter and a suggestion was applied.
    pub fn handle_input(&mut self, ui: &mut egui::Ui, suggestions: &[Suggestion]) -> bool {
        let groups = self.grouped_suggestions(suggestions);
        let flattened: Vec<&Suggestion> =
            groups.iter().flat_map(|group| &group.suggestions).collect();

        // Ensure the selection is always valid and defaults to the first item.
        if self
            .selected_suggestion
            .is_none_or(|index| index >= flattened.len())
        {
            self.selected_suggestion = flattened.first().map(|_| 0);
        }

        if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.selected_suggestion = Some(
                self.selected_suggestion
                    .map(|index| (index + 1) % flattened.len())
                    .unwrap_or(0),
            );
        } else if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.selected_suggestion = Some(
                self.selected_suggestion
                    .map(|index| index.saturating_sub(1))
                    .unwrap_or(flattened.len().saturating_sub(1)),
            );
        } else if ui.input(|i| i.key_pressed(egui::Key::Enter))
            && let Some(index) = self.selected_suggestion
            && let Some(suggestion) = flattened.get(index)
        {
            self.apply_suggestion(suggestion);
            self.selected_suggestion = None;
            return true;
        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.selected_suggestion = None;
        } else if ui.input(|i| i.key_pressed(egui::Key::Backspace)) && self.query.is_empty() {
            // Backspace on empty input also signals dismissal, matching Zed-like
            // behavior.
            self.selected_suggestion = None;
        }

        false
    }

    /// Returns `true` when the user pressed Escape (or Backspace on an empty
    /// query), indicating the caller should close the search modal.
    ///
    /// `just_applied` is typically the result of [`Self::handle_input`]; when
    /// `true`, Enter was just pressed and the modal should also close.
    pub fn wants_close(&self, ui: &egui::Ui, just_applied: bool) -> bool {
        just_applied
            || ui.input(|i| i.key_pressed(egui::Key::Escape))
            || (self.query.is_empty() && ui.input(|i| i.key_pressed(egui::Key::Backspace)))
    }

    /// Builds the display label for a suggestion based on its kind.
    fn suggestion_label(&self, suggestion: &Suggestion) -> egui::RichText {
        match suggestion.kind {
            SuggestionKind::Sort => egui::RichText::new(&suggestion.text).monospace(),
            SuggestionKind::Author => egui::RichText::new(format!("@author:{}", suggestion.text)),
            SuggestionKind::Filter => egui::RichText::new(format!("#{}", suggestion.text)),
            SuggestionKind::Plain => egui::RichText::new(&suggestion.text),
        }
    }

    /// Returns suggestions filtered by the current query, grouped by kind.
    fn grouped_suggestions(&self, suggestions: &[Suggestion]) -> Vec<SuggestionGroup> {
        let needle = self.active_needle().to_lowercase();
        let max_per_group = 8;

        let filter_kind = |kind: SuggestionKind| {
            suggestions
                .iter()
                .filter(|s| s.kind == kind && s.text.to_lowercase().contains(&needle))
                .take(max_per_group)
                .cloned()
                .collect::<Vec<_>>()
        };

        vec![
            SuggestionGroup {
                title: "name:",
                suggestions: filter_kind(SuggestionKind::Plain),
            },
            SuggestionGroup {
                title: "tag:",
                suggestions: filter_kind(SuggestionKind::Filter),
            },
            SuggestionGroup {
                title: "author:",
                suggestions: filter_kind(SuggestionKind::Author),
            },
            SuggestionGroup {
                title: "sort:",
                suggestions: filter_kind(SuggestionKind::Sort),
            },
        ]
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
    fn active_needle_detects_hash() {
        let mut bar = SearchBar {
            query: "screw #Ble".to_owned(),
            id: egui::Id::new("test_search_bar"),
            request_focus: false,
            selected_suggestion: None,
        };
        assert_eq!(bar.active_needle(), "Ble");

        bar.query = "@sort:down".to_owned();
        assert_eq!(bar.active_needle(), "sort:down");

        bar.query = "parametric".to_owned();
        assert_eq!(bar.active_needle(), "parametric");
    }

    #[test]
    fn apply_sort_replaces_partial_token() {
        let mut bar = SearchBar {
            query: "screw @so".to_owned(),
            id: egui::Id::new("test_search_bar"),
            request_focus: false,
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
            request_focus: false,
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
            request_focus: false,
            selected_suggestion: None,
        };
        bar.apply_suggestion(&Suggestion::filter("Blender"));
        assert_eq!(bar.query, "#Blender");
    }

    #[test]
    fn apply_author_replaces_partial_token() {
        let mut bar = SearchBar {
            query: "screw @Zen".to_owned(),
            id: egui::Id::new("test_search_bar"),
            request_focus: false,
            selected_suggestion: None,
        };
        bar.apply_suggestion(&Suggestion::author("ZenFlow"));
        assert_eq!(bar.query, "screw @author:ZenFlow");
    }

    #[test]
    fn apply_author_replaces_lonely_partial_token() {
        let mut bar = SearchBar {
            query: "@Zen".to_owned(),
            id: egui::Id::new("test_search_bar"),
            request_focus: false,
            selected_suggestion: None,
        };
        bar.apply_suggestion(&Suggestion::author("ZenFlow"));
        assert_eq!(bar.query, "@author:ZenFlow");
    }
}
