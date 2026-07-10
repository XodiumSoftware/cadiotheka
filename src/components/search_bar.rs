//! Search bar UI widget for the Cadiotheka hub.
//!
//! This file only handles rendering and user interaction. All query parsing,
//! filtering, scoring, and suggestion logic lives in [`crate::search_engine`].

use crate::engines::{ParsedQuery, Suggestion, SuggestionKind};
use crate::i18n;

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
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        suggestions: &[Suggestion],
        parse: impl FnOnce(&str) -> ParsedQuery,
    ) -> ParsedQuery {
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
                    .changed()
                    .then_some(egui::SetOpenCommand::Bool(true)),
            )
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .layout(egui::Layout::top_down_justified(egui::Align::Min))
            .width(response.rect.width())
            .frame(egui::Frame::new().inner_margin(0.0));

        let popup_width = response.rect.width();
        popup.show(|ui| {
            ui.set_min_width(popup_width);
            ui.set_max_width(popup_width);
            self.render_suggestions(ui, suggestions)
        });

        parse(&self.query)
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
