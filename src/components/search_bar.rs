//! Search bar widget for the Cadiotheka hub.

use crate::i18n;
use egui_phosphor_icons::icons;

/// State and rendering for a search control bar.
#[derive(Default)]
pub struct SearchBar {
    /// Current search query.
    pub query: String,
}

impl SearchBar {
    /// Draw the search bar inside a card-like container.
    ///
    /// Returns the current query.
    pub fn show(&mut self, ui: &mut egui::Ui) -> &str {
        let margin = 24.0;
        let mut frame = egui::Frame::group(ui.style());
        frame.fill = ui.visuals().panel_fill;
        frame.shadow = egui::Shadow {
            offset: [0, 6],
            blur: 8,
            spread: 0,
            color: ui.visuals().window_shadow.color,
        };

        egui::Frame::new()
            .inner_margin(egui::Margin::same(margin as i8))
            .show(ui, |ui| {
                frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(icons::MAGNIFYING_GLASS.regular().size(14.0));
                        ui.add(
                            egui::TextEdit::singleline(&mut self.query)
                                .hint_text(i18n::SearchBar::PLACEHOLDER)
                                .margin(egui::vec2(12.0, 6.0)),
                        );
                    });
                });
            });

        self.query.as_str()
    }
}
